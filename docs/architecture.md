# Architecture

The workspace is split into a library crate and a CLI crate.

## Library

`crates/textcase` exposes a small public API centered around:

- `CaseOptions`
- `CaseMode`
- `GermanMode`
- `convert`
- `sentence_case`
- `sentence_case_title`
- `PluginSet`

Internally, the library is divided into:

- `tokenize`: token model and lightweight tokenization
- `icu`: locale-aware casing helpers and segmentation wrappers
- `case`: sentence/title conversion and subtitle normalization
- `lang`: language profile registry and German heuristics
- `plugin`: schema, metadata, and inspection types
- `lexicon`: prepared/plugin types, JSON/FST loading, and mergeable provider implementation

## CLI

`crates/textcase-cli` reuses the library’s prepared/plugin types and adds:

- source registry metadata
- fetch and prepare workflows
- JSON/FST plugin construction
- plugin inspection

The CLI keeps licensing metadata close to the source registry so `list-sources`, `show-license`, `prepare`, and `inspect-plugin` share one descriptor model.
