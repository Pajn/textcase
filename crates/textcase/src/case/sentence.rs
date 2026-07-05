use std::collections::HashSet;

use crate::{
    case::{
        mode_is_sentence_like, mode_is_title, prepare_input, should_capitalize_after_separator,
        should_keep_lowercase_in_title,
    },
    config::{CaseMode, CaseOptions},
    icu::{capitalize_word_locale, lowercase_locale, titlecase_word_locale},
    lang::{german, profile_for_locale},
    lexicon::{builtin_canonical_form, builtin_canonical_phrase},
    tokenize::{Token, TokenKind, is_sentence_terminal, reconstruct, tokenize},
    util::{is_all_caps, is_mixed_case},
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

    for (index, token) in tokens.iter_mut().enumerate() {
        match token.kind {
            TokenKind::Word => {
                let original = token.text.clone();
                let lower = lowercase_locale(&original, options.locale);
                let at_sentence_cap = sentence_start
                    || (after_subtitle && options.capitalize_after_subtitle_separator);
                let is_edge = edge_words.contains(&index);
                let recase_context = RecaseContext {
                    should_capitalize: at_sentence_cap,
                    is_edge_word: is_edge,
                    previous_word: previous_word.as_deref(),
                    previous_word2: previous_word2.as_deref(),
                };
                token.text = if (options.preserve_acronyms && is_all_caps(&original))
                    || (options.preserve_mixed_case && is_mixed_case(&original))
                {
                    original
                } else if options.preserve_known_proper_nouns {
                    lookup_word(options, &lower).unwrap_or_else(|| {
                        recase_word(&original, &lower, options, profile, recase_context)
                    })
                } else {
                    recase_word(&original, &lower, options, profile, recase_context)
                };
                sentence_start = false;
                after_subtitle = false;
                previous_word2 = previous_word.take();
                previous_word = Some(lower);
            }
            TokenKind::Punctuation => {
                if is_sentence_terminal(&token.text) {
                    sentence_start = true;
                }
                if should_capitalize_after_separator(
                    options.capitalize_after_subtitle_separator,
                    &token.text,
                ) {
                    after_subtitle = true;
                }
            }
            TokenKind::Whitespace | TokenKind::Symbol => {}
        }
    }

    if options.preserve_known_proper_nouns {
        apply_phrase_replacements(&mut tokens, options);
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
    if options.locale.starts_with("de")
        && let Some(restored) = german::recase_token(
            original,
            lower,
            recase_context.previous_word,
            recase_context.previous_word2,
            options.german_mode,
            options.lexicons,
        )
    {
        return restored;
    }

    if mode_is_title(options.mode) {
        if should_keep_lowercase_in_title(profile, lower, recase_context.is_edge_word) {
            lowercase_locale(original, options.locale)
        } else {
            titlecase_word_locale(original, options.locale)
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

fn lookup_word(options: &CaseOptions<'_>, lower: &str) -> Option<String> {
    if let Some(builtin) = builtin_canonical_form(lower) {
        return Some(builtin.to_string());
    }
    options
        .lexicons
        .and_then(|provider| provider.canonical_form(options.locale, lower))
}

fn apply_phrase_replacements(tokens: &mut [Token], options: &CaseOptions<'_>) {
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
