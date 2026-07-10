# textcase (library)

Multilingual sentence and title recasing for Latin-script languages.

`textcase` recases text whose capitalization is wrong or missing — lowercase feeds, SHOUTED titles, Title Cased Prose — while preserving capitalization that carries information. It works without any external data; optional lexicon plugins add proper-noun restoration on top.

```toml
[dependencies]
textcase = "0.2"
```

## Quickstart

```rust
use textcase::{sentence_case, sentence_case_title};

// Sentence case: one capital per sentence, meaningful casing preserved.
assert_eq!(
    sentence_case("yesterday Alice met Bob in Paris. we had fun", "en"),
    "Yesterday Alice met Bob in Paris. We had fun"
);

// Sentence-title mode: like sentence case, but a subtitle separator
// (":", " - ", "—") starts a new capitalized segment and line breaks flatten.
assert_eq!(
    sentence_case_title("the album - remastered", "en"),
    "The album - Remastered"
);
```

Every knob lives on `CaseOptions`; `convert` is the full-control entry point:

```rust
use textcase::{CaseMode, CaseOptions, SubtitleSeparatorStyle, convert};

let mut options = CaseOptions::for_locale("en");
options.mode = CaseMode::Title;
assert_eq!(convert("the lord of the rings", &options), "The Lord of the Rings");

let mut options = CaseOptions::for_locale("en");
options.mode = CaseMode::SentenceTitle;
options.subtitle_separator_style = SubtitleSeparatorStyle::ColonSpace;
assert_eq!(
    convert("the rise of github - inside rust tooling", &options),
    "The rise of GitHub: Inside rust tooling"
);
```

## Modes

| `CaseMode` | Behavior |
| --- | --- |
| `Sentence` | Capitalizes sentence starts, lowercases the rest (minus preserved casing). A colon in prose does not capitalize (`"note: this is fine"`). Line breaks are kept. |
| `SentenceTitle` | Sentence case for single-line titles: subtitle separators start a new capitalized segment, and line breaks flatten to spaces. |
| `Title` | English-style title case: every word capitalized except per-language stop words and particles; the first and last words and subtitle openers are always capitalized. |

## What conversion does

**Sentence boundaries.** `.`, `!`, `?` and CJK/Arabic/Devanagari terminals end sentences. Periods after decimals (`3.5`), initials (`J. K.`), and known abbreviations do not — abbreviations are classified per language: titles (`Dr.`, `vs.`) never end a sentence, numeric forms (`No.`, `vol.`) only abbreviate before a number, and phrase-final forms (`etc.`, `Inc.`) yield to a capitalized next word. An ellipsis (`…`, `...`) ends a sentence only when the input already capitalizes the next word.

**Preserved casing.** Unless disabled through options:

- acronyms — all-caps words like `NASA` (unless the whole sentence is shouted)
- mixed case — `iPhone`, `McDonald`, `LaTeX`
- existing capitals — a capitalized mid-sentence word (`Alice`) is an unknown proper noun and keeps its capital; capitals in fully shouted or title-cased sentences carry no signal and are still normalized
- known proper nouns — a small builtin lexicon (`github` → `GitHub`) plus any plugins you load; user lexicons override the builtin entries

**Shouting detection** is per sentence: `"BREAKING NEWS. the NASA probe landed"` converts the first sentence and keeps the acronym in the second.

**English orthography.** The pronoun `i` and its contractions (`i'm`, `i'll`, …) are always capitalized in English locales.

**Subtitle separators.** `subtitle_separator_style` rewrites between `:`, ` - `, and ` — ` styles; numeric and single-letter ranges (`3 - 5`, `a - z`) and unspaced colons (`10:30`, `re:invent`) are left alone.

## Options

`CaseOptions::default()` targets `"en"`, `CaseMode::Sentence`, and everything preserved:

| Field | Default | Meaning |
| --- | --- | --- |
| `locale` | `"en"` | BCP 47-ish tag; the primary language selects the profile and ICU rules |
| `mode` | `Sentence` | see the modes table |
| `subtitle_separator_style` | `Preserve` | normalize subtitle separators to a single style |
| `capitalize_after_subtitle_separator` | `true` | capitalize the word opening a subtitle (title modes only) |
| `preserve_acronyms` | `true` | keep all-caps words outside shouted sentences |
| `preserve_mixed_case` | `true` | keep internal capitals (`iPhone`) |
| `preserve_known_proper_nouns` | `true` | apply builtin and plugin canonical forms |
| `preserve_existing_capitals` | `true` | keep capitalized mid-sentence words in sentence modes |
| `normalize_whitespace` | `true` | collapse whitespace runs (line breaks survive plain sentence mode) |
| `german_mode` | `Conservative` | German noun-recovery tier, see below |
| `lexicons` | `None` | a `LexiconProvider` such as `PluginSet` |

## Languages

Dedicated profiles: English, German, French, Spanish, Portuguese, Italian, Dutch, Swedish, Danish, Norwegian, Finnish, Turkish, Azerbaijani, and Lithuanian. A profile contributes stop words and lowercase particles (title mode), abbreviation classes (sentence splitting), contraction tails (`don't` vs `O'Brien`), and elision prefixes (French/Italian `l'`, `d'`, `qu'` stay lowercase in titles: `d'affaires` → `d'Affaires`).

Any other locale gets a neutral profile that assumes nothing beyond a few Latin abbreviations. Character-level casing is always locale-aware through ICU: Turkish `istanbul` → `İstanbul`, Dutch `ijsselmeer` → `IJsselmeer`, Greek `ΟΔΟΣ` → `οδος`.

German additionally recovers noun capitalization in tiers (`german_mode`): `Conservative` does none, `Balanced` adds article/preposition/suffix heuristics (`"ich mag die wissenschaft"` → `"Ich mag die Wissenschaft"`), `Aggressive` adds ranked-candidate plugin data. Details in [docs/german.md](https://github.com/Pajn/textcase/blob/main/docs/german.md).

## Lexicon plugins

Plugins restore canonical forms the input cannot express — proper nouns, brands, multi-word names (`van der waals` → `van der Waals`). Load one or more into a `PluginSet`:

```rust,no_run
use textcase::{CaseOptions, PluginSet, convert};

let plugins = PluginSet::from_fst_path("data/plugins/geonames-de.tclx")?;
let mut options = CaseOptions::for_locale("de");
options.lexicons = Some(&plugins);
println!("{}", convert("wir fliegen nach köln", &options));
# Ok::<(), textcase::Error>(())
```

`PluginSet::from_json_bytes` loads the JSON container; `merge` combines sets with later entries winning. Plugins are produced by the [`textcase-cli`](https://github.com/Pajn/textcase/blob/main/crates/textcase-cli/README.md) from public data sources — see [docs/sources.md](https://github.com/Pajn/textcase/blob/main/docs/sources.md) for choosing one and [docs/plugin-format.md](https://github.com/Pajn/textcase/blob/main/docs/plugin-format.md) for the container formats.

## Analysis

`convert_analyze` (and its `sentence_case_analyze` sugar) return a `CaseAnalysis` alongside the recased string: an overall `Confidence` and a `CasingSpan` per edit recording the deciding `CasingRule`, its confidence, and whether it changed. The output is byte-identical to `convert`; both share one cascade.

Confidence has three tiers. `Solid` is a structural rule (sentence start, stop-word lowering, plain lowercasing), an explicit lexicon match, or a structural transform. `Unverified` is an ordinary word capitalized as a title word with no lexicon to confirm it is not a name or brand — the open-world case. `Heuristic` is a call that could genuinely be wrong: acronym-versus-word classification, keeping a lone capital as a proper noun, or the German noun heuristics. The analysis's confidence is the most concerning tier across every span, so callers can flag `Heuristic` results for review.

```rust
use textcase::{sentence_case_analyze, CasingRule, Confidence};

let input = "the NASA probe landed";
let analysis = sentence_case_analyze(input, "en");
assert_eq!(analysis.output, "The NASA probe landed");
// Preserving "NASA" as an acronym is a heuristic, so the whole result is flagged.
assert_eq!(analysis.confidence, Confidence::Heuristic);

// `source` ranges index your input; `output` ranges index the output. Filter on
// `changed` for just the edits.
let changed: Vec<_> = analysis
    .spans
    .iter()
    .filter(|span| span.changed)
    .map(|span| (&analysis.output[span.output.clone()], span.rule))
    .collect();
assert_eq!(changed, vec![("The", CasingRule::SentenceStart)]);

// Every span maps back to the original bytes you passed in.
let first = &analysis.spans[0];
assert_eq!(&input[first.source.clone()], "the");
```

`span.source` ranges index the raw input you passed in, so keep that string to resolve them — normalization never shifts the offsets. Structural edits are reported too: a collapsed whitespace run or a rewritten subtitle separator (`" - "` → `": "`) surfaces as a `WhitespaceCollapsed` or `SeparatorNormalized` span, so `spans` fully reconstructs the input-to-output diff.

## More

- runnable examples: [`examples/`](https://github.com/Pajn/textcase/tree/main/crates/textcase/examples)
- benchmarks and current numbers: [docs/performance.md](https://github.com/Pajn/textcase/blob/main/docs/performance.md)
- module layout for contributors: [docs/architecture.md](https://github.com/Pajn/textcase/blob/main/docs/architecture.md)
