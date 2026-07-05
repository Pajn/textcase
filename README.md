# textcase

Multilingual sentence and title recasing for Latin-script languages.

`textcase` takes text with wrong or missing capitalization — lowercase feeds, SHOUTED titles, Title Cased Prose — and recases it the way the target language would write it. It is conservative by design: capitalization that carries information (acronyms, `iPhone`-style casing, mid-sentence proper nouns) is preserved, and optional lexicon plugins restore canonical forms the input has lost.

```rust
use textcase::sentence_case;

assert_eq!(
    sentence_case("BREAKING NEWS. the NASA probe landed", "en"),
    "Breaking news. The NASA probe landed"
);
```

The workspace ships two crates:

- [`textcase`](crates/textcase/) — the library: conversion modes, language profiles, and plugin loading. Start with its [README](crates/textcase/README.md).
- [`textcase-cli`](crates/textcase-cli/) — the `textcase` binary for building and inspecting lexicon plugins from public data sources. It is tooling for plugin production, not a text-recasing command. Start with its [README](crates/textcase-cli/README.md).

## What it covers

- **Three modes**: sentence case, sentence case with capitalized subtitles (`Title: Like this`), and full title case with per-language stop words.
- **Sentence boundary detection** that understands abbreviations (`Dr.`, `No. 5`, `usw.`), decimals, initials, ellipses, and CJK/Arabic/Devanagari terminals.
- **Preservation of meaningful casing**: acronyms (`NASA`), mixed case (`iPhone`, `McDonald`), and capitalized mid-sentence words (`Alice`) survive conversion; fully shouted or title-cased input is still normalized.
- **Language profiles** for English, German, French, Spanish, Portuguese, Italian, Dutch, Swedish, Danish, Norwegian, Finnish, Turkish, Azerbaijani, and Lithuanian — stop words, particles, abbreviations, and elision rules per language, with a neutral fallback for everything else. Locale-aware casing itself (Turkish `İ`, Dutch `IJ`, Greek final sigma) goes through ICU.
- **German noun recovery** in three tiers (conservative, balanced, aggressive) — see [docs/german.md](docs/german.md).
- **Lexicon plugins** (JSON or FST) that restore canonical proper-noun forms (`github` → `GitHub`, `new york` → `New York`) from sources you choose and license-check yourself.

Out of scope: grammar-aware analysis, non-Latin-script recasing (CJK terminals only delimit sentences), and markup handling — feed it plain text.

## Quickstart

Library ([full guide](crates/textcase/README.md)):

```rust
use textcase::{CaseMode, CaseOptions, SubtitleSeparatorStyle, convert, sentence_case};

assert_eq!(
    sentence_case("the rise of github - inside rust tooling", "en"),
    "The rise of GitHub - inside rust tooling"
);

let options = CaseOptions {
    locale: "en",
    mode: CaseMode::SentenceTitle,
    subtitle_separator_style: SubtitleSeparatorStyle::ColonSpace,
    ..CaseOptions::default()
};
assert_eq!(
    convert("the rise of github - inside rust tooling", &options),
    "The rise of GitHub: Inside rust tooling"
);
```

CLI ([full guide](crates/textcase-cli/README.md)):

```bash
cargo binstall textcase-cli   # or: cargo install textcase-cli

textcase lexicon list-sources
textcase lexicon fetch geonames --country DE --output-dir data/raw
textcase lexicon prepare geonames --input data/raw/geonames-de.tsv \
    --output data/prepared/geonames-de.json --kind canonical-map --lang de
textcase lexicon build-plugin data/prepared/geonames-de.json --format fst \
    --output data/plugins/geonames-de.tclx
```

## Documentation map

| Read | When you want to |
| --- | --- |
| [crates/textcase/README.md](crates/textcase/README.md) | use the library: modes, options, behavior, loading plugins |
| [crates/textcase-cli/README.md](crates/textcase-cli/README.md) | build lexicon plugins with the CLI |
| [docs/sources.md](docs/sources.md) | choose a data source and follow its fetch/prepare workflow |
| [docs/german.md](docs/german.md) | understand the German heuristic tiers |
| [docs/plugin-format.md](docs/plugin-format.md) | read or emit the JSON/FST plugin containers yourself |
| [docs/licensing-policy.md](docs/licensing-policy.md) | understand the green/yellow/orange source classes |
| [docs/architecture.md](docs/architecture.md) | contribute: how the crates and modules fit together |
| [docs/performance.md](docs/performance.md) | run the benchmarks and see current numbers |

## Workspace layout

- `crates/textcase`: core library
- `crates/textcase-cli`: CLI tooling (`textcase` binary)
- `docs/`: topic guides (see the map above)
- `examples/`: top-level usage examples mirroring the crate examples
- `tests/fixtures/`: fixture space for integration and e2e inputs

## Development

Current MSRV is `1.88`. The workspace exposes no optional Cargo features; new features should only be added when they clearly preserve a usable zero-plugin default and keep licensing boundaries explicit.

- format with `cargo fmt --all`
- lint with `cargo clippy --workspace --all-targets -- -D warnings`
- test with `cargo test --workspace`
- CI runs the same checks on pushes and pull requests
