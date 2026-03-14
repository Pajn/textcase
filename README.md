# textcase

`textcase` is a Rust workspace for multilingual sentence/title recasing in Latin-script languages.

Current MSRV is `1.85`. The workspace currently exposes no optional Cargo features; new features should only be added when they clearly preserve a usable zero-plugin default and keep licensing boundaries explicit.

It provides:

- a `textcase` library with sentence/title conversion, locale-aware casing helpers, language profiles, German heuristic modes, and pluggable lexicons
- a `textcase` CLI for listing sources, showing licensing guidance, fetching upstream or sample data, preparing lexicons, building JSON/FST plugins, and inspecting plugin metadata
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
textcase lexicon show-license wikidata
textcase lexicon fetch wikidata --lang en --sample --output-dir data/raw
textcase lexicon prepare wikidata --input data/raw/wikidata-en.json --output data/prepared/wikidata-en.json --kind canonical-map --lang en
textcase lexicon build-plugin data/prepared/wikidata-en.json --format json --output data/plugins/wikidata-en.json
textcase lexicon inspect-plugin data/plugins/wikidata-en.json
```

Use `--sample` when you want deterministic local fixtures for tests or offline examples.

For real upstream downloads, `fetch` currently has built-in workflows for `geonames` and `ud-german-gsd`. Other sources currently require either an explicit `--url` or `--sample`; they no longer silently pretend to fetch production data.

## Source classes

- `green`: clean redistribution story (`wikidata`, `gnd`, `orcid`, `musicbrainz`)
- `yellow`: attribution guidance required (`geonames`, `getty`)
- `orange`: stronger obligations and opt-in-only workflows (`wiktionary`, `dbpedia`, `openstreetmap`, `ud-german-gsd`)
- `gray`: experimental / lower-priority sources (`omw`)

## Development guardrails

- format with `cargo fmt --all`
- lint with `cargo clippy --workspace --all-targets -- -D warnings`
- test with `cargo test --workspace`
- CI runs the same checks on pushes and pull requests

## German modes

- `Conservative`: sentence starts, subtitle starts, lexicon restoration, acronym protection
- `Balanced`: adds noun-context heuristics
- `Aggressive`: adds ranked-candidate plugin integration on top of balanced heuristics
