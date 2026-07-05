mod normalize;
mod sentence;
mod subtitle;
mod title;

use crate::config::{CaseMode, CaseOptions};

pub use sentence::{convert, sentence_case, sentence_case_title};

pub(crate) fn mode_is_title(mode: CaseMode) -> bool {
    matches!(mode, CaseMode::Title)
}

pub(crate) fn mode_is_sentence_like(mode: CaseMode) -> bool {
    matches!(mode, CaseMode::Sentence | CaseMode::SentenceTitle)
}

/// Whether the mode treats a subtitle separator as the start of a new
/// capitalized segment. Plain sentence case does not: a colon in running prose
/// ("note: this is fine") is not a new sentence.
pub(crate) fn mode_capitalizes_after_subtitle(mode: CaseMode) -> bool {
    matches!(mode, CaseMode::Title | CaseMode::SentenceTitle)
}

pub(crate) use normalize::{normalize_subtitle_separators, normalize_whitespace};
pub(crate) use subtitle::should_capitalize_after_separator;
pub(crate) use title::should_keep_lowercase_in_title;

pub(crate) fn prepare_input(input: &str, options: &CaseOptions<'_>) -> String {
    let whitespace = if options.normalize_whitespace {
        normalize_whitespace(input)
    } else {
        input.to_string()
    };
    normalize_subtitle_separators(&whitespace, options.subtitle_separator_style)
}
