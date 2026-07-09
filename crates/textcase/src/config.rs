use crate::lexicon::LexiconProvider;

/// Selects the conversion strategy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CaseMode {
    Sentence,
    Title,
    SentenceTitle,
}

/// Controls subtitle separator normalization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SubtitleSeparatorStyle {
    Preserve,
    ColonSpace,
    SpaceDashSpace,
    EmDashSpace,
}

/// Controls the German heuristic level.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub enum GermanMode {
    Conservative,
    Balanced,
    Aggressive,
}

/// Controls conversion behavior.
///
/// Construct with [`CaseOptions::for_locale`] (or [`CaseOptions::default`]) and
/// then set the fields you need; the struct is `#[non_exhaustive]` so that new
/// options can be added without breaking existing callers:
///
/// ```
/// use textcase::{CaseMode, CaseOptions};
///
/// let mut options = CaseOptions::for_locale("de");
/// options.mode = CaseMode::Title;
/// ```
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct CaseOptions<'a> {
    pub locale: &'a str,
    pub mode: CaseMode,
    pub subtitle_separator_style: SubtitleSeparatorStyle,
    pub capitalize_after_subtitle_separator: bool,
    pub preserve_acronyms: bool,
    pub preserve_mixed_case: bool,
    pub preserve_known_proper_nouns: bool,
    /// Keeps a capitalized mid-sentence word ("Alice") capitalized in the
    /// sentence modes. Capitals carry no signal in shouted or fully
    /// title-cased sentences, so those are still recased.
    pub preserve_existing_capitals: bool,
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
            preserve_existing_capitals: true,
            normalize_whitespace: true,
            german_mode: GermanMode::Conservative,
            lexicons: None,
        }
    }
}
