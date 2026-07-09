mod ambiguity;
mod contexts;
mod heuristics;
mod rules;

use super::LanguageProfile;

pub use rules::recase_token;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        stop_words: &[
            "am", "an", "auf", "der", "die", "das", "ein", "eine", "für", "im", "in", "mit",
            "oder", "und", "von", "zu",
        ],
        lowercase_particles: &[
            "am", "an", "auf", "der", "die", "das", "ein", "eine", "im", "in", "mit", "von", "zu",
        ],
        title_abbreviations: &["dr", "prof", "hr", "fr", "st"],
        numeric_abbreviations: &["nr", "bd", "ca"],
        trailing_abbreviations: &["usw", "bzw", "ggf", "evtl", "etc"],
        ..LanguageProfile::neutral()
    }
}
