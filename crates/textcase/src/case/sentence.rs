use std::collections::{HashMap, HashSet};

use crate::{
    analysis::{CaseAnalysis, CasingRule, CasingSpan, Confidence},
    case::{
        mode_capitalizes_after_subtitle, mode_flattens_lines, mode_is_sentence_like, mode_is_title,
        normalize_separator_tokens, normalize_whitespace_tokens, should_keep_lowercase_in_title,
        subtitle_separator_flags,
    },
    config::{CaseMode, CaseOptions},
    icu::{
        capitalize_word_locale, lowercase_locale, primary_language, titlecase_word_locale,
        uppercase_first_grapheme,
    },
    lang::{english_always_capitalized, german, profile_for_locale},
    lexicon::{builtin_canonical_form, builtin_canonical_phrase, builtin_form_is_ambiguous},
    tokenize::{
        AbbreviationKind, Token, TokenKind, is_sentence_terminal, is_wide_sentence_terminal,
        reconstruct, tokenize,
    },
    util::{is_acronym_candidate, is_mixed_case},
};

#[derive(Clone, Copy)]
struct RecaseContext<'a> {
    should_capitalize: bool,
    is_edge_word: bool,
    previous_word: Option<&'a str>,
    previous_word2: Option<&'a str>,
}

/// Converts text according to the provided locale, case mode, and lexicon settings.
pub fn convert(input: &str, options: &CaseOptions<'_>) -> String {
    run::<false>(input, options).output
}

/// Converts `input` like [`convert`] and additionally returns a [`CaseAnalysis`]:
/// the recased string, an overall [`Confidence`], and a [`CasingSpan`] per word
/// recording the deciding [`CasingRule`], its confidence, and whether it changed.
///
/// The output is byte-identical to [`convert`] on the same input and options;
/// both share one per-word cascade.
///
/// ```
/// use textcase::{convert_analyze, CaseOptions, CasingRule, Confidence};
///
/// let analysis = convert_analyze("the rise of github", &CaseOptions::for_locale("en"));
/// assert_eq!(analysis.output, "The rise of GitHub");
/// // The canonical-form restore and sentence-start capital are both solid.
/// assert_eq!(analysis.confidence, Confidence::Solid);
/// assert!(analysis.spans.iter().any(|span| span.rule == CasingRule::CanonicalLexicon));
/// ```
#[must_use]
pub fn convert_analyze(input: &str, options: &CaseOptions<'_>) -> CaseAnalysis {
    run::<true>(input, options)
}

/// Converts `input` to sentence case for `locale` and returns a [`CaseAnalysis`].
/// Sugar for [`convert_analyze`] with default options; see it for details.
#[must_use]
pub fn sentence_case_analyze(input: &str, locale: &str) -> CaseAnalysis {
    convert_analyze(input, &CaseOptions::for_locale(locale))
}

/// The shared conversion core. `RECORD` is a compile-time flag: [`convert`]
/// instantiates it `false`, so the span bookkeeping below is dead-code
/// eliminated and the hot path is unchanged; [`convert_analyze`] instantiates it
/// `true` to record per-word attribution.
fn run<const RECORD: bool>(input: &str, options: &CaseOptions<'_>) -> CaseAnalysis {
    // Tokenize the raw input directly and normalize at the token level, so every
    // token keeps a `source` range back into `input` regardless of how its text
    // is later rewritten.
    let mut tokens = tokenize(input);
    if tokens.is_empty() {
        return CaseAnalysis {
            output: input.to_string(),
            confidence: Confidence::Solid,
            spans: Vec::new(),
        };
    }

    if options.normalize_whitespace {
        normalize_whitespace_tokens(&mut tokens, mode_flattens_lines(options.mode));
    }

    let profile = profile_for_locale(options.locale);
    let sentence_boundaries = sentence_boundary_flags(&tokens, options.locale, profile);
    let subtitle_separators = subtitle_separator_flags(&tokens);
    // When a whole sentence is capitalized it is a shouted title, not a
    // sequence of acronyms, so acronym preservation must not block conversion.
    let sentence_ids = token_sentence_ids(&sentence_boundaries);
    let sentence_shouting =
        sentence_shouting_flags(&tokens, &sentence_ids, profile, options.locale);
    let sentence_title_like =
        sentence_title_like_flags(&tokens, &sentence_ids, profile, options.locale);
    let word_indices: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| token.is_word().then_some(index))
        .collect();
    let edge_words: HashSet<usize> = word_indices
        .first()
        .into_iter()
        .chain(word_indices.last())
        .copied()
        .collect();

    // Rewrite subtitle separators to the requested style. The flags were read
    // above, before this changes any punctuation text.
    let separator_rewrites = normalize_separator_tokens(
        &mut tokens,
        &subtitle_separators,
        options.subtitle_separator_style,
    );

    // Per-word-token rule: which cascade branch decided it. Byte ranges come from
    // each token's own `source`, so no offsets are threaded here. Only the
    // analysis path allocates.
    let mut word_rules: Vec<Option<CasingRule>> = if RECORD {
        vec![None; tokens.len()]
    } else {
        Vec::new()
    };

    let mut sentence_start = true;
    let mut after_subtitle = false;
    let mut previous_word: Option<String> = None;
    let mut previous_word2: Option<String> = None;
    let mut sentence_start_words: HashSet<usize> = HashSet::new();

    for (index, token) in tokens.iter_mut().enumerate() {
        match token.kind {
            TokenKind::Word => {
                // Move the word out rather than clone: token.text is reassigned
                // below before the arm ends, so the empty placeholder left here
                // is never observed.
                let original = std::mem::take(&mut token.text);
                let lower = lowercase_locale(&original, options.locale);
                let at_sentence_cap = sentence_start
                    || (after_subtitle
                        && options.capitalize_after_subtitle_separator
                        && mode_capitalizes_after_subtitle(options.mode));
                if at_sentence_cap {
                    sentence_start_words.insert(index);
                }
                let is_edge = edge_words.contains(&index);
                let recase_context = RecaseContext {
                    should_capitalize: at_sentence_cap,
                    is_edge_word: is_edge,
                    previous_word: previous_word.as_deref(),
                    previous_word2: previous_word2.as_deref(),
                };
                let shouting = sentence_shouting[sentence_ids[index]];
                let title_like = sentence_title_like[sentence_ids[index]];
                let (text, rule) = decide_word(
                    original,
                    &lower,
                    options,
                    profile,
                    recase_context,
                    shouting,
                    title_like,
                );
                token.text = text;
                if RECORD {
                    word_rules[index] = Some(rule);
                }
                sentence_start = false;
                after_subtitle = false;
                previous_word2 = previous_word.take();
                previous_word = Some(lower);
            }
            TokenKind::Punctuation => {
                if sentence_boundaries[index] {
                    sentence_start = true;
                }
                if options.capitalize_after_subtitle_separator && subtitle_separators[index] {
                    after_subtitle = true;
                }
                // Punctuation breaks an article/preposition-to-noun bond, so the
                // German noun heuristic must not read context across it.
                previous_word = None;
                previous_word2 = None;
            }
            TokenKind::Symbol => {
                previous_word = None;
                previous_word2 = None;
            }
            TokenKind::Whitespace => {}
        }
    }

    let merges = if options.preserve_known_proper_nouns {
        apply_phrase_replacements(&mut tokens, options, &sentence_start_words)
    } else {
        Vec::new()
    };

    let output = reconstruct(&tokens);
    if !RECORD {
        return CaseAnalysis {
            output,
            confidence: Confidence::Solid,
            spans: Vec::new(),
        };
    }
    let spans = build_spans(
        input,
        &output,
        &tokens,
        &word_rules,
        &merges,
        &separator_rewrites,
    );
    let confidence = spans.iter().fold(Confidence::Solid, |worst, span| {
        worst.most_concerning(span.confidence)
    });
    CaseAnalysis {
        output,
        confidence,
        spans,
    }
}

/// Decides one word's casing and returns the produced text plus the rule behind
/// it. The single source of truth for the per-word cascade; the returned rule is
/// a compile-time constant per branch, so it costs nothing on the plain path,
/// where the caller discards it.
fn decide_word(
    original: String,
    lower: &str,
    options: &CaseOptions<'_>,
    profile: crate::lang::LanguageProfile,
    recase_context: RecaseContext<'_>,
    shouting: bool,
    title_like: bool,
) -> (String, CasingRule) {
    // A known canonical form wins over acronym/mixed-case preservation, so
    // "GITHUB" becomes "GitHub"; an all-caps word absent from the lexicon
    // ("NASA") is still preserved. Titles always carry the signal; elsewhere the
    // input must have cased the word itself, in a sentence where capitals mean
    // something.
    let casing_signal =
        mode_is_title(options.mode) || (original != *lower && !shouting && !title_like);
    if options.preserve_known_proper_nouns
        && let Some(canonical) = lookup_word(options, lower, casing_signal)
    {
        return (canonical, CasingRule::CanonicalLexicon);
    }
    if options.preserve_acronyms && !shouting && is_acronym_candidate(&original) {
        return (original, CasingRule::AcronymPreserved);
    }
    if options.preserve_mixed_case && is_mixed_case(&original) {
        return (original, CasingRule::MixedCasePreserved);
    }
    if options.preserve_existing_capitals
        && mode_is_sentence_like(options.mode)
        && !recase_context.should_capitalize
        && !shouting
        && !title_like
        && is_simple_capitalized(&original)
    {
        // A lone capitalized word in an otherwise lowercase sentence is an
        // unknown proper noun; lowercasing it would destroy information no
        // lexicon can restore.
        return (original, CasingRule::ProperNounPreserved);
    }
    recase_word(&original, lower, options, profile, recase_context)
}

/// Assembles the final spans from raw token ranges. A multiword-lexicon phrase
/// and a rewritten subtitle separator each collapse to one span covering their
/// tokens; every word contributes a span (changed or not); a whitespace token
/// contributes one only when normalization altered it. `source` ranges come from
/// each token's raw `source`; `output` ranges from cumulative output positions.
fn build_spans(
    input: &str,
    output: &str,
    tokens: &[Token],
    word_rules: &[Option<CasingRule>],
    phrase_merges: &[(usize, usize)],
    separator_rewrites: &[(usize, usize)],
) -> Vec<CasingSpan> {
    // First-token index -> (last-token index, rule) for the merged transforms,
    // and the interior tokens they absorb (so those emit no separate span).
    let mut merged: HashMap<usize, (usize, CasingRule)> = HashMap::new();
    let mut absorbed: HashSet<usize> = HashSet::new();
    for &(first, last) in phrase_merges {
        merged.insert(first, (last, CasingRule::MultiwordLexicon));
        absorbed.extend((first + 1)..=last);
    }
    for &(first, last) in separator_rewrites {
        merged.insert(first, (last, CasingRule::SeparatorNormalized));
        absorbed.extend((first + 1)..=last);
    }

    // Cumulative output byte offset of each token, so a merged span can reach the
    // end of its last token.
    let mut out_starts = Vec::with_capacity(tokens.len());
    let mut acc = 0;
    for token in tokens {
        out_starts.push(acc);
        acc += token.text.len();
    }
    let out_end_of = |index: usize| out_starts[index] + tokens[index].text.len();

    let mut spans = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if absorbed.contains(&index) {
            continue;
        }
        let (source, output_range, rule) = if let Some(&(last, rule)) = merged.get(&index) {
            (
                token.source.start..tokens[last].source.end,
                out_starts[index]..out_end_of(last),
                rule,
            )
        } else if let Some(rule) = word_rules[index] {
            (
                token.source.clone(),
                out_starts[index]..out_end_of(index),
                rule,
            )
        } else if matches!(token.kind, TokenKind::Whitespace)
            && input[token.source.clone()] != token.text
        {
            (
                token.source.clone(),
                out_starts[index]..out_end_of(index),
                CasingRule::WhitespaceCollapsed,
            )
        } else {
            continue;
        };
        let changed = input[source.clone()] != output[output_range.clone()];
        // A separator already in the requested style is not a transformation, so
        // do not report it; other rules (including WhitespaceCollapsed) still
        // record their spans whether or not the text changed.
        if rule == CasingRule::SeparatorNormalized && !changed {
            continue;
        }
        spans.push(CasingSpan {
            source,
            output: output_range,
            rule,
            confidence: rule.confidence(),
            changed,
        });
    }
    spans
}

/// Converts text to sentence case with default options for the given locale.
///
/// ```
/// use textcase::sentence_case;
///
/// assert_eq!(sentence_case("the rise of github", "en"), "The rise of GitHub");
/// ```
pub fn sentence_case(input: &str, locale: &str) -> String {
    let options = CaseOptions {
        locale,
        ..CaseOptions::default()
    };
    convert(input, &options)
}

/// Converts text to sentence-title mode with default subtitle capitalization rules.
pub fn sentence_case_title(input: &str, locale: &str) -> String {
    let options = CaseOptions {
        locale,
        mode: CaseMode::SentenceTitle,
        ..CaseOptions::default()
    };
    convert(input, &options)
}

fn recase_word(
    original: &str,
    lower: &str,
    options: &CaseOptions<'_>,
    profile: crate::lang::LanguageProfile,
    recase_context: RecaseContext<'_>,
) -> (String, CasingRule) {
    if primary_language(options.locale) == "de"
        && let Some(restored) = german::recase_token(
            original,
            lower,
            recase_context.previous_word,
            recase_context.previous_word2,
            options.german_mode,
            options.lexicons,
        )
    {
        // The German recase decides letter case for the word body, but a
        // sentence- or subtitle-initial word (and title edge words) must still
        // start with a capital, so compose the two rather than short-circuit.
        let needs_capital = (mode_is_sentence_like(options.mode)
            && recase_context.should_capitalize)
            || (mode_is_title(options.mode)
                && (recase_context.should_capitalize || recase_context.is_edge_word));
        let text = if needs_capital {
            uppercase_first_grapheme(&restored, options.locale)
        } else {
            restored
        };
        return (text, CasingRule::GermanNoun);
    }

    if mode_is_title(options.mode) {
        // A word that opens the title or a subtitle segment is always
        // capitalized, even when it is a stop word ("Something: The Reckoning").
        if recase_context.should_capitalize
            || !should_keep_lowercase_in_title(profile, lower, recase_context.is_edge_word)
        {
            let cased = titlecase_word_locale(
                original,
                options.locale,
                profile.contraction_tails,
                profile.elision_prefixes,
            );
            // An elided particle stays lowercase mid-title ("d'Affaires"),
            // but a title-opening or edge word still starts with a capital
            // ("L'Homme").
            if recase_context.should_capitalize {
                (
                    uppercase_first_grapheme(&cased, options.locale),
                    CasingRule::SentenceStart,
                )
            } else if recase_context.is_edge_word {
                (
                    uppercase_first_grapheme(&cased, options.locale),
                    CasingRule::TitleEdge,
                )
            } else {
                (cased, CasingRule::Capitalized)
            }
        } else {
            (
                lowercase_locale(original, options.locale),
                CasingRule::SmallWord,
            )
        }
    } else if mode_is_sentence_like(options.mode) {
        let always_capitalized =
            primary_language(options.locale) == "en" && english_always_capitalized(lower);
        if recase_context.should_capitalize {
            (
                capitalize_word_locale(original, options.locale),
                CasingRule::SentenceStart,
            )
        } else if always_capitalized {
            (
                capitalize_word_locale(original, options.locale),
                CasingRule::AlwaysCapitalized,
            )
        } else {
            (
                lowercase_locale(original, options.locale),
                CasingRule::Lowercased,
            )
        }
    } else {
        (
            lowercase_locale(original, options.locale),
            CasingRule::Lowercased,
        )
    }
}

/// Marks which punctuation tokens are true sentence terminals.
///
/// A terminal that is immediately followed by an alphanumeric character (the
/// internal dots of `e.g.` or `3.5`) does not start a new sentence, and a
/// period directly after an abbreviation or a single-letter initial is skipped
/// as well.
fn sentence_boundary_flags(
    tokens: &[Token],
    locale: &str,
    profile: crate::lang::LanguageProfile,
) -> Vec<bool> {
    let mut flags = vec![false; tokens.len()];
    for index in 0..tokens.len() {
        let token = &tokens[index];
        if !matches!(token.kind, TokenKind::Punctuation) || !is_sentence_terminal(&token.text) {
            continue;
        }

        // Non-Latin terminals are unambiguous and not space-separated, so the
        // "followed by alphanumeric" guard (for "3.5"/"e.g.") must not apply.
        if is_wide_sentence_terminal(&token.text) {
            flags[index] = true;
            continue;
        }

        // An ellipsis usually trails off mid-sentence rather than ending it,
        // so it only starts a new sentence when the input already capitalizes
        // the next word.
        if token.text == "…" || is_ellipsis_period(tokens, index) {
            if tokens.get(index + 1).is_some_and(|next| next.text == ".") {
                // Interior dot of a "..." run; the run's last dot decides.
                continue;
            }
            flags[index] = next_word_is_capitalized(tokens, index);
            continue;
        }

        if token.text == "." {
            // Only the period is ambiguous with decimals ("3.5") and internal
            // abbreviation dots ("e.g."); "!" and "?" end the sentence even
            // without a following space.
            let followed_by_alphanumeric = tokens
                .get(index + 1)
                .is_some_and(|next| next.text.chars().next().is_some_and(char::is_alphanumeric));
            if followed_by_alphanumeric {
                continue;
            }

            if index > 0 && matches!(tokens[index - 1].kind, TokenKind::Word) {
                let previous = lowercase_locale(&tokens[index - 1].text, locale);
                let suppressed = match profile.abbreviation_kind(&previous) {
                    Some(AbbreviationKind::Title) => true,
                    Some(AbbreviationKind::Numeric) => next_word_starts_with_digit(tokens, index),
                    Some(AbbreviationKind::Trailing) => !next_word_is_capitalized(tokens, index),
                    None => is_single_letter(&previous),
                };
                if suppressed {
                    continue;
                }
            }
        }

        flags[index] = true;
    }
    flags
}

/// Assigns each token the index of the sentence it belongs to; a boundary
/// terminal closes its own sentence.
fn token_sentence_ids(boundaries: &[bool]) -> Vec<usize> {
    let mut ids = Vec::with_capacity(boundaries.len());
    let mut current = 0;
    for &boundary in boundaries {
        ids.push(current);
        if boundary {
            current += 1;
        }
    }
    ids
}

/// Whether each sentence is written in capitals (a shouted title) rather than
/// containing isolated acronyms.
///
/// A sentence is shouting when every word is all-caps, or when the only
/// lowercase words are stop words and at least one all-caps word has five or
/// more letters. That converts "NEW YORK vs THE WORLD" while keeping the short
/// all-caps words of "USA vs USSR" as acronyms.
fn sentence_shouting_flags(
    tokens: &[Token],
    sentence_ids: &[usize],
    profile: crate::lang::LanguageProfile,
    locale: &str,
) -> Vec<bool> {
    let sentence_count = sentence_ids.last().map_or(0, |last| last + 1);
    let mut all_caps = vec![true; sentence_count];
    let mut caps_or_stop_word = vec![true; sentence_count];
    let mut has_caps_word = vec![false; sentence_count];
    let mut has_long_caps_word = vec![false; sentence_count];

    for (index, token) in tokens.iter().enumerate() {
        if !token.is_word() || !token.text.chars().any(char::is_alphabetic) {
            continue;
        }
        let id = sentence_ids[index];
        let word_is_all_caps = !token.text.chars().any(char::is_lowercase);
        if word_is_all_caps {
            has_caps_word[id] = true;
            let letters = token.text.chars().filter(|ch| ch.is_alphabetic()).count();
            if letters >= 5 {
                has_long_caps_word[id] = true;
            }
        } else {
            all_caps[id] = false;
            let lower = lowercase_locale(&token.text, locale);
            if !profile.keeps_lowercase_in_title(&lower)
                && !profile.keeps_particle_lowercase(&lower)
            {
                caps_or_stop_word[id] = false;
            }
        }
    }

    (0..sentence_count)
        .map(|id| {
            has_caps_word[id] && (all_caps[id] || (caps_or_stop_word[id] && has_long_caps_word[id]))
        })
        .collect()
}

/// Whether each sentence looks like title-cased input: every word after the
/// sentence-initial one either carries a capital or is a stop word, with at
/// least two carrying capitals. In such input capitalization is a formatting
/// artifact, not a proper-noun signal, so nothing is preserved from it.
fn sentence_title_like_flags(
    tokens: &[Token],
    sentence_ids: &[usize],
    profile: crate::lang::LanguageProfile,
    locale: &str,
) -> Vec<bool> {
    let sentence_count = sentence_ids.last().map_or(0, |last| last + 1);
    let mut saw_initial_word = vec![false; sentence_count];
    let mut rest_conforms = vec![true; sentence_count];
    let mut capitalized_words = vec![0usize; sentence_count];

    for (index, token) in tokens.iter().enumerate() {
        if !token.is_word() || !token.text.chars().any(char::is_alphabetic) {
            continue;
        }
        let id = sentence_ids[index];
        if !saw_initial_word[id] {
            saw_initial_word[id] = true;
            continue;
        }
        if token.text.chars().any(char::is_uppercase) {
            capitalized_words[id] += 1;
        } else {
            let lower = lowercase_locale(&token.text, locale);
            if !profile.keeps_lowercase_in_title(&lower)
                && !profile.keeps_particle_lowercase(&lower)
            {
                rest_conforms[id] = false;
            }
        }
    }

    (0..sentence_count)
        .map(|id| rest_conforms[id] && capitalized_words[id] >= 2)
        .collect()
}

/// A word like "Alice": a leading capital, at least two letters, and no
/// further capitals (internal capitals are mixed case, handled separately).
fn is_simple_capitalized(word: &str) -> bool {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_uppercase() && chars.clone().any(char::is_alphabetic) && !chars.any(char::is_uppercase)
}

/// A period that belongs to a `..`/`...` run is ellipsis punctuation, not a
/// full stop.
fn is_ellipsis_period(tokens: &[Token], index: usize) -> bool {
    tokens[index].text == "."
        && (index
            .checked_sub(1)
            .is_some_and(|previous| tokens[previous].text == ".")
            || tokens.get(index + 1).is_some_and(|next| next.text == "."))
}

/// Whether the input already capitalizes the first word after `index`,
/// skipping whitespace and intervening punctuation such as quotes.
fn next_word_is_capitalized(tokens: &[Token], index: usize) -> bool {
    tokens[index + 1..]
        .iter()
        .find(|token| token.is_word())
        .is_some_and(|token| token.text.chars().next().is_some_and(char::is_uppercase))
}

fn next_word_starts_with_digit(tokens: &[Token], index: usize) -> bool {
    tokens[index + 1..]
        .iter()
        .find(|token| token.is_word())
        .is_some_and(|token| token.text.chars().next().is_some_and(char::is_numeric))
}

fn is_single_letter(word: &str) -> bool {
    let mut chars = word.chars();
    matches!((chars.next(), chars.next()), (Some(first), None) if first.is_alphabetic())
}

/// Looks up a canonical form, letting a user lexicon override the builtin
/// entries. An ambiguous builtin form ("rust") is only restored when the
/// input carried a casing signal for the word.
fn lookup_word(options: &CaseOptions<'_>, lower: &str, casing_signal: bool) -> Option<String> {
    if let Some(from_provider) = options
        .lexicons
        .and_then(|provider| provider.canonical_form(options.locale, lower))
    {
        return Some(from_provider);
    }
    let builtin = builtin_canonical_form(lower)?;
    if builtin_form_is_ambiguous(lower) && !casing_signal {
        return None;
    }
    Some(builtin.to_string())
}

/// Rewrites multiword lexicon phrases in place and returns, for the analysis
/// path, the `(first_token, last_token)` word-token index pairs it collapsed so
/// their spans can merge. The plain path discards the return value.
fn apply_phrase_replacements(
    tokens: &mut [Token],
    options: &CaseOptions<'_>,
    sentence_start_words: &HashSet<usize>,
) -> Vec<(usize, usize)> {
    let word_indices: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| token.is_word().then_some(index))
        .collect();

    let mut merges = Vec::new();
    let mut cursor = 0;
    while cursor < word_indices.len() {
        let mut matched = None;
        for span_len in (2..=4).rev() {
            if cursor + span_len > word_indices.len() {
                continue;
            }
            let span = &word_indices[cursor..cursor + span_len];
            if !contains_only_words_and_spaces(tokens, span[0], *span.last().unwrap()) {
                continue;
            }
            let phrase = build_phrase_key(tokens, span, options.locale);
            // The user lexicon overrides the builtin phrases as well.
            let canonical = options
                .lexicons
                .and_then(|provider| provider.canonical_phrase(options.locale, &phrase))
                .or_else(|| builtin_canonical_phrase(&phrase).map(str::to_owned));
            if let Some(canonical) = canonical {
                matched = Some((span_len, canonical));
                break;
            }
        }

        if let Some((span_len, canonical)) = matched {
            let first = word_indices[cursor];
            let last = word_indices[cursor + span_len - 1];
            // A canonical phrase can begin with a lowercase particle ("van der
            // Waals"); force a capital when the span starts a sentence.
            let canonical = if sentence_start_words.contains(&first) {
                uppercase_first_grapheme(&canonical, options.locale)
            } else {
                canonical
            };
            tokens[first].text = canonical;
            for token in &mut tokens[first + 1..=last] {
                token.text.clear();
            }
            merges.push((first, last));
            cursor += span_len;
        } else {
            cursor += 1;
        }
    }

    merges
}

fn build_phrase_key(tokens: &[Token], span: &[usize], locale: &str) -> String {
    span.iter()
        .map(|index| lowercase_locale(&tokens[*index].text, locale))
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_only_words_and_spaces(tokens: &[Token], start: usize, end: usize) -> bool {
    tokens[start..=end]
        .iter()
        .all(|token| matches!(token.kind, TokenKind::Word | TokenKind::Whitespace))
}
