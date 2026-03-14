use crate::{config::GermanMode, icu::capitalize_word_locale, lexicon::LexiconProvider};

use super::heuristics;

pub fn recase_token(
    token: &str,
    lower: &str,
    previous: Option<&str>,
    previous2: Option<&str>,
    mode: GermanMode,
    lexicons: Option<&dyn LexiconProvider>,
) -> Option<String> {
    if mode >= GermanMode::Aggressive {
        if let Some(provider) = lexicons {
            if let Some(candidates) = provider.ranked_candidates("de", lower) {
                if let Some(candidate) = candidates
                    .into_iter()
                    .max_by(|left, right| left.score.total_cmp(&right.score))
                {
                    return Some(candidate.value);
                }
            }
        }
    }

    if mode >= GermanMode::Balanced
        && heuristics::looks_like_noun_context(lower, previous, previous2)
    {
        return Some(capitalize_word_locale(token, "de"));
    }

    None
}
