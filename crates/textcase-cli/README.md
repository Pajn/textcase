# textcase-cli

Command-line tooling for building and inspecting [`textcase`](https://crates.io/crates/textcase) lexicon plugins.

The binary is named `textcase`, but it does **not** recase text — that is the library's job. This tool produces the optional plugin files the library loads at runtime: it fetches public proper-noun and lexical data, converts it into prepared lexicons, packages those as JSON or FST plugins, and keeps license metadata attached the whole way.

## Install

Prebuilt binaries (Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64) via [`cargo binstall`](https://github.com/cargo-bins/cargo-binstall) — note the crate to install is `textcase-cli`, the binary it ships is `textcase`:

```bash
cargo binstall textcase-cli
```

Or build from source:

```bash
cargo install textcase-cli
```

## The pipeline

Every plugin is produced in three steps, each with an inspectable intermediate file:

```text
upstream source          data/raw/…              data/prepared/…            data/plugins/…
(GeoNames, Wikidata, …) ── fetch ──> raw dump ── prepare ──> lexicon JSON ── build-plugin ──> .json / .tclx
```

All commands live under `textcase lexicon`:

| Command | What it does |
| --- | --- |
| `list-sources` | catalog of supported sources with their license class |
| `show-license <source>` | licensing and workflow guidance for one source |
| `fetch <source>` | download raw data (built-in workflow or `--url`-driven) |
| `prepare <source>` | convert a raw dump into a prepared lexicon (`--kind`, `--lang`) |
| `build-plugin <prepared>` | package a prepared lexicon as a plugin (`--format json\|fst`) |
| `inspect-plugin <path>` | print a plugin's metadata, license, and payload summary |

## Worked example: German place names

```bash
# 1. See what exists and check the license story first.
textcase lexicon list-sources
textcase lexicon show-license geonames

# 2. Fetch the raw dump (GeoNames has a built-in workflow; omit --country
#    for the global dataset).
textcase lexicon fetch geonames --country DE --output-dir data/raw

# 3. Convert it into a prepared lexicon.
textcase lexicon prepare geonames \
    --input data/raw/geonames-de.tsv \
    --output data/prepared/geonames-de.json \
    --kind canonical-map --lang de

# 4. Package it. FST is the compact runtime format; JSON stays greppable.
textcase lexicon build-plugin data/prepared/geonames-de.json \
    --format fst --output data/plugins/geonames-de.tclx

# 5. Verify what you built.
textcase lexicon inspect-plugin data/plugins/geonames-de.tclx
```

Then load it from the library:

```rust,ignore
let plugins = textcase::PluginSet::from_fst_path("data/plugins/geonames-de.tclx")?;
```

## Sources and licensing

Sources are classed by their redistribution story — `green` (clean: `wikidata`, `gnd`, `orcid`, `musicbrainz`), `yellow` (attribution guidance: `geonames`, `getty`), and `orange` (stronger obligations, strictly opt-in: `wiktionary`, `dbpedia`, `openstreetmap`, `ud-german-gsd`). Orange sources refuse to fetch or prepare until you pass their acknowledgement flag (`--acknowledge-share-alike`, `--acknowledge-odbl`, or `--acknowledge-cc-by-sa`), and the obligation is recorded in the plugin metadata.

`fetch` has built-in workflows for `geonames`, `wiktionary`, and `ud-german-gsd`, which have stable dataset-style downloads. The other sources are query-oriented APIs, so you pass the exact upstream slice you want with `--url`.

**[docs/sources.md](https://github.com/Pajn/textcase/blob/main/docs/sources.md) is the full catalog** — what each source provides, when to use it, which `--kind` to prepare, and a copy-pasteable command sequence per source. Related reading:

- [docs/plugin-format.md](https://github.com/Pajn/textcase/blob/main/docs/plugin-format.md) — the JSON and FST container formats
- [docs/licensing-policy.md](https://github.com/Pajn/textcase/blob/main/docs/licensing-policy.md) — how the source classes are assigned
