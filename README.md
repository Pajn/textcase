# textcase

`textcase` is a Rust workspace for multilingual sentence/title recasing in Latin-script languages.

Current MSRV is `1.85`. The workspace currently exposes no optional Cargo features; new features should only be added when they clearly preserve a usable zero-plugin default and keep licensing boundaries explicit.

It provides:

- a `textcase` library with sentence/title conversion, locale-aware casing helpers, language profiles, German heuristic modes, and pluggable lexicons
- a `textcase` CLI for listing sources, showing licensing guidance, fetching upstream data, preparing lexicons, building JSON/FST plugins, and inspecting plugin metadata
- explicit licensing boundaries so the core crate remains usable with zero external data while optional plugins add better proper-noun and lexical recovery

## Workspace layout

- `crates/textcase`: core library
- `crates/textcase-cli`: CLI tooling (`textcase` binary)
- `docs/`: architecture, plugin, source, licensing, German, and performance notes
- `examples/`: top-level usage examples mirroring the crate examples
- `tests/fixtures/`: fixture space for integration and e2e inputs

## Library quickstart

```rust
use textcase::{sentence_case, CaseMode, CaseOptions, SubtitleSeparatorStyle, convert};

let plain = sentence_case("the rise of github - inside rust tooling", "en");
assert_eq!(plain, "The rise of GitHub - inside Rust tooling");

let options = CaseOptions {
    locale: "en",
    mode: CaseMode::SentenceTitle,
    subtitle_separator_style: SubtitleSeparatorStyle::ColonSpace,
    ..CaseOptions::default()
};
assert_eq!(convert("the rise of github - inside rust tooling", &options), "The rise of GitHub: Inside Rust tooling");
```

## CLI quickstart

```bash
textcase lexicon list-sources
textcase lexicon show-license geonames
textcase lexicon fetch geonames --country DE --output-dir data/raw
textcase lexicon prepare geonames --input data/raw/geonames-de.tsv --output data/prepared/geonames-de.json --kind canonical-map --lang de
textcase lexicon build-plugin data/prepared/geonames-de.json --format fst --output data/plugins/geonames-de.tclx
textcase lexicon inspect-plugin data/plugins/geonames-de.tclx
```

For opt-in lexical support, a complete built-in Wiktionary workflow is also available:

```bash
textcase lexicon show-license wiktionary
textcase lexicon fetch wiktionary --lang de --acknowledge-share-alike --output-dir data/raw
textcase lexicon prepare wiktionary --input data/raw/wiktionary-de.jsonl.gz --output data/prepared/wiktionary-de.json --kind word-set --lang de --acknowledge-share-alike
```

`fetch` has built-in workflows for `geonames`, `ud-german-gsd`, and `wiktionary`.

The remaining sources are intentionally URL-driven because their upstreams are query-oriented APIs rather than single canonical dumps. The setup guide in `docs/sources.md` explains what each source provides, when to use it, and the exact endpoint style to pass with `--url`.

## Choosing a source

Start with the cleanly redistributable proper-noun sources:

- `wikidata` for multilingual entities and aliases
- `gnd` for authority-style German and European names
- `orcid` for researcher names
- `musicbrainz` for artists, albums, labels, and works

Add domain-specific sources as needed:

- `geonames` for country- or world-scale place names
- `getty` for art, heritage, and museum vocabularies
- `openstreetmap` for street- and locality-level geography
- `wiktionary` for lexical hints and inflected forms
- `ud-german-gsd` for German ranked-candidate enrichment

The full source selection and setup guide lives in `docs/sources.md`.

## Source classes

- `green`: clean redistribution story (`wikidata`, `gnd`, `orcid`, `musicbrainz`)
- `yellow`: attribution guidance required (`geonames`, `getty`)
- `orange`: stronger obligations and opt-in-only workflows (`wiktionary`, `dbpedia`, `openstreetmap`, `ud-german-gsd`)

## Development guardrails

- format with `cargo fmt --all`
- lint with `cargo clippy --workspace --all-targets -- -D warnings`
- test with `cargo test --workspace`
- CI runs the same checks on pushes and pull requests

## German modes

- `Conservative`: sentence starts, subtitle starts, lexicon restoration, acronym protection
- `Balanced`: adds noun-context heuristics
- `Aggressive`: adds ranked-candidate plugin integration on top of balanced heuristics
