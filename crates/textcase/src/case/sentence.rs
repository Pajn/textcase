use std::collections::HashSet;

use crate::{
    case::{
        mode_capitalizes_after_subtitle, mode_is_sentence_like, mode_is_title, prepare_input,
        should_keep_lowercase_in_title, subtitle_separator_flags,
    },
    config::{CaseMode, CaseOptions},
    icu::{
        capitalize_word_locale, lowercase_locale, primary_language, titlecase_word_locale,
        uppercase_first_grapheme,
    },
    lang::{german, profile_for_locale},
    lexicon::{builtin_canonical_form, builtin_canonical_phrase},
    tokenize::{
        AbbreviationKind, Token, TokenKind, abbreviation_kind, is_sentence_terminal,
        is_wide_sentence_terminal, reconstruct, tokenize,
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
    let prepared = prepare_input(input, options);
    let mut tokens = tokenize(&prepared);
    if tokens.is_empty() {
        return prepared;
    }

    let profile = profile_for_locale(options.locale);
    let sentence_boundaries = sentence_boundary_flags(&tokens, options.locale);
    let subtitle_separators = subtitle_separator_flags(&tokens);
    // When a whole sentence is capitalized it is a shouted title, not a
    // sequence of acronyms, so acronym preservation must not block conversion.
    let sentence_ids = token_sentence_ids(&sentence_boundaries);
    let sentence_shouting =
        sentence_shouting_flags(&tokens, &sentence_ids, profile, options.locale);
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
                // A known canonical form wins over acronym/mixed-case
                // preservation, so "GITHUB" becomes "GitHub"; an all-caps word
                // absent from the lexicon ("NASA") is still preserved.
                let canonical = options
                    .preserve_known_proper_nouns
                    .then(|| lookup_word(options, &lower));
                token.text = if let Some(Some(canonical)) = canonical {
                    canonical
                } else if (options.preserve_acronyms
                    && !sentence_shouting[sentence_ids[index]]
                    && is_acronym_candidate(&original))
                    || (options.preserve_mixed_case && is_mixed_case(&original))
                {
                    original
                } else {
                    recase_word(&original, &lower, options, profile, recase_context)
                };
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

    if options.preserve_known_proper_nouns {
        apply_phrase_replacements(&mut tokens, options, &sentence_start_words);
    }

    reconstruct(&tokens)
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
) -> String {
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
        return if needs_capital {
            uppercase_first_grapheme(&restored, options.locale)
        } else {
            restored
        };
    }

    if mode_is_title(options.mode) {
        // A word that opens the title or a subtitle segment is always
        // capitalized, even when it is a stop word ("Something: The Reckoning").
        if recase_context.should_capitalize
            || !should_keep_lowercase_in_title(profile, lower, recase_context.is_edge_word)
        {
            titlecase_word_locale(original, options.locale)
        } else {
            lowercase_locale(original, options.locale)
        }
    } else if mode_is_sentence_like(options.mode) {
        if recase_context.should_capitalize {
            capitalize_word_locale(original, options.locale)
        } else {
            lowercase_locale(original, options.locale)
        }
    } else {
        lowercase_locale(original, options.locale)
    }
}

/// Marks which punctuation tokens are true sentence terminals.
///
/// A terminal that is immediately followed by an alphanumeric character (the
/// internal dots of `e.g.` or `3.5`) does not start a new sentence, and a
/// period directly after an abbreviation or a single-letter initial is skipped
/// as well.
fn sentence_boundary_flags(tokens: &[Token], locale: &str) -> Vec<bool> {
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
                let suppressed = match abbreviation_kind(&previous) {
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

fn lookup_word(options: &CaseOptions<'_>, lower: &str) -> Option<String> {
    if let Some(builtin) = builtin_canonical_form(lower) {
        return Some(builtin.to_string());
    }
    options
        .lexicons
        .and_then(|provider| provider.canonical_form(options.locale, lower))
}

fn apply_phrase_replacements(
    tokens: &mut [Token],
    options: &CaseOptions<'_>,
    sentence_start_words: &HashSet<usize>,
) {
    let word_indices: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| token.is_word().then_some(index))
        .collect();

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
            let canonical = builtin_canonical_phrase(&phrase)
                .map(str::to_owned)
                .or_else(|| {
                    options
                        .lexicons
                        .and_then(|provider| provider.canonical_phrase(options.locale, &phrase))
                });
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
            cursor += span_len;
        } else {
            cursor += 1;
        }
    }
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
