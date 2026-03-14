use crate::lexicon::LexiconProvider;

/// Selects the conversion strategy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaseMode {
    Sentence,
    Title,
    SentenceTitle,
}

/// Controls subtitle separator normalization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubtitleSeparatorStyle {
    Preserve,
    ColonSpace,
    SpaceDashSpace,
    EmDashSpace,
}

/// Controls the German heuristic level.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GermanMode {
    Conservative,
    Balanced,
    Aggressive,
}

/// Controls conversion behavior.
#[derive(Clone, Copy)]
pub struct CaseOptions<'a> {
    pub locale: &'a str,
    pub mode: CaseMode,
    pub subtitle_separator_style: SubtitleSeparatorStyle,
    pub capitalize_after_subtitle_separator: bool,
    pub preserve_acronyms: bool,
    pub preserve_mixed_case: bool,
    pub preserve_known_proper_nouns: bool,
    pub normalize_whitespace: bool,
    pub german_mode: GermanMode,
    pub lexicons: Option<&'a dyn LexiconProvider>,
}

impl<'a> CaseOptions<'a> {
    pub fn for_locale(locale: &'a str) -> Self {
        Self {
            locale,
            ..CaseOptions::default()
        }
    }
}

impl Default for CaseOptions<'static> {
    fn default() -> Self {
        Self {
            locale: "en",
            mode: CaseMode::Sentence,
            subtitle_separator_style: SubtitleSeparatorStyle::Preserve,
            capitalize_after_subtitle_separator: true,
            preserve_acronyms: true,
            preserve_mixed_case: true,
            preserve_known_proper_nouns: true,
            normalize_whitespace: true,
            german_mode: GermanMode::Conservative,
            lexicons: None,
        }
    }
}
