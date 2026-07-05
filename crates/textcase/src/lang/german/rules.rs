use crate::{config::GermanMode, icu::capitalize_word_locale, lexicon::LexiconProvider};

use super::heuristics;

/// Ranked candidates below this score are noise, not evidence. The German UD
/// source scores nouns at 2.0+ and proper nouns at 3.0+, with sub-1.0 values
/// only reachable through feature bonuses on non-noun readings.
const MIN_RANKED_SCORE: f32 = 1.0;

pub fn recase_token(
    token: &str,
    lower: &str,
    previous: Option<&str>,
    previous2: Option<&str>,
    mode: GermanMode,
    lexicons: Option<&dyn LexiconProvider>,
) -> Option<String> {
    if mode >= GermanMode::Aggressive
        && let Some(provider) = lexicons
        && let Some(candidates) = provider.ranked_candidates("de", lower)
        && let Some(candidate) = candidates
            .into_iter()
            .filter(|candidate| candidate.score >= MIN_RANKED_SCORE)
            .max_by(|left, right| left.score.total_cmp(&right.score))
    {
        return Some(candidate.value);
    }

    if mode >= GermanMode::Balanced
        && heuristics::looks_like_noun_context(lower, previous, previous2)
    {
        return Some(capitalize_word_locale(token, "de"));
    }

    None
}
