//! Rich analysis output: per-word casing decisions and their confidence.
//!
//! [`crate::convert_analyze`] returns a [`CaseAnalysis`] alongside the recased
//! string, recording which rule decided each word and how much to trust it. The
//! plain [`crate::convert`] path shares the same cascade, so the two can never
//! disagree on output (guarded by a parity test).

use core::ops::Range;

/// How much to trust a casing decision.
///
/// The tiers are ordered by concern for review: [`Confidence::Solid`] is safest,
/// [`Confidence::Heuristic`] most warrants a human look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Confidence {
    /// A structural rule (sentence start, stop-word lowering, plain lowercasing)
    /// or an explicit lexicon match. Not a guess.
    Solid,
    /// An ordinary word capitalized as a title word with no lexicon to confirm it
    /// is not a name or brand that should be spelled differently. Correct under
    /// the title rules, but the open-world "a lexicon might disagree" case applies.
    Unverified,
    /// A heuristic that could genuinely be wrong: acronym-versus-word
    /// classification, keeping a lone capital as a proper noun, or the German
    /// noun-capitalization heuristics.
    Heuristic,
}

impl Confidence {
    fn rank(self) -> u8 {
        match self {
            Confidence::Solid => 0,
            Confidence::Unverified => 1,
            Confidence::Heuristic => 2,
        }
    }

    /// Returns the more concerning (less confident) of two tiers.
    #[must_use]
    pub fn most_concerning(self, other: Self) -> Self {
        if other.rank() > self.rank() {
            other
        } else {
            self
        }
    }
}

/// The rule that decided a word's casing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CasingRule {
    /// Restored from a single-word canonical lexicon or the built-in list
    /// (`github` -> `GitHub`).
    CanonicalLexicon,
    /// Restored from a multiword-phrase lexicon (`new york` -> `New York`).
    MultiwordLexicon,
    /// Left all-caps as a likely acronym (`NASA`). (heuristic)
    AcronymPreserved,
    /// Left unchanged because the input already carried internal capitals
    /// (`iPhone`, `McDonald`).
    MixedCasePreserved,
    /// A lone capitalized word in an otherwise lowercase sentence, kept as a
    /// likely proper noun that no lexicon could restore. (heuristic)
    ProperNounPreserved,
    /// Recased by a German noun-capitalization heuristic (balanced context rule
    /// or the aggressive ranked-lexicon guess). (heuristic)
    GermanNoun,
    /// Capitalized as the first word of a sentence or of a subtitle segment.
    SentenceStart,
    /// Capitalized because the language always capitalizes it (English `I`).
    AlwaysCapitalized,
    /// Capitalized as the first or last word of a title.
    TitleEdge,
    /// Lowercased as a title stop word (article, conjunction, short preposition).
    SmallWord,
    /// Lowercased as an ordinary word in a sentence body or a non-title context.
    Lowercased,
    /// Capitalized as an ordinary title word with no lexicon confirmation.
    /// Correct under the title rules, but a name or brand might disagree.
    Capitalized,
    /// A subtitle separator rewritten to the requested style (` - ` -> `: `).
    /// A structural edit, not a casing decision.
    SeparatorNormalized,
    /// A whitespace run collapsed to a single space or newline, or trimmed from
    /// the edges of the input. A structural edit, not a casing decision.
    WhitespaceCollapsed,
}

impl CasingRule {
    /// How much to trust this rule's decision.
    #[must_use]
    pub fn confidence(self) -> Confidence {
        match self {
            CasingRule::CanonicalLexicon
            | CasingRule::MultiwordLexicon
            | CasingRule::MixedCasePreserved
            | CasingRule::SentenceStart
            | CasingRule::AlwaysCapitalized
            | CasingRule::TitleEdge
            | CasingRule::SmallWord
            | CasingRule::Lowercased
            | CasingRule::SeparatorNormalized
            | CasingRule::WhitespaceCollapsed => Confidence::Solid,

            CasingRule::AcronymPreserved
            | CasingRule::ProperNounPreserved
            | CasingRule::GermanNoun => Confidence::Heuristic,

            CasingRule::Capitalized => Confidence::Unverified,
        }
    }
}

/// One edit's record: the rule behind it, how much to trust it, and whether it
/// actually changed the text.
///
/// `source` is a byte range into the raw input you passed to
/// [`crate::convert_analyze`]; `output` is a byte range into
/// [`CaseAnalysis::output`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CasingSpan {
    /// Byte range in the raw input.
    pub source: Range<usize>,
    /// Byte range of the produced word in [`CaseAnalysis::output`].
    pub output: Range<usize>,
    /// The rule that decided this word's casing.
    pub rule: CasingRule,
    /// How much to trust the decision.
    pub confidence: Confidence,
    /// Whether the output text differs from the source text. `false` means the
    /// rule confirmed the input was already correct (still useful to know a
    /// heuristic was involved).
    pub changed: bool,
}

/// The result of [`crate::convert_analyze`]: the recased string, a span for
/// every word, and an overall confidence.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CaseAnalysis {
    /// The recased output. Byte-identical to [`crate::convert`] on the same input
    /// and options. [`CasingSpan::source`] ranges index the raw input you passed
    /// in, so keep that string around to resolve them.
    pub output: String,
    /// The least-confident tier across every span's decision;
    /// [`Confidence::Solid`] when there are none.
    pub confidence: Confidence,
    /// The edits from `source` to `output`, in reading order: one span per word
    /// (a multiword-lexicon phrase collapses to one), plus a span for each
    /// structural transform ([`CasingRule::SeparatorNormalized`],
    /// [`CasingRule::WhitespaceCollapsed`]). Word spans appear whether or not the
    /// word changed; transform spans only where something changed. Filter on
    /// [`CasingSpan::changed`] for just the edits.
    pub spans: Vec<CasingSpan>,
}
