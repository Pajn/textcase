# Source catalog and setup guide

Every plugin should have a clear origin, a clear license story, and a workflow a new user can follow without reading the implementation. This guide covers all ten supported sources; the CLI pipeline itself (fetch → prepare → build-plugin) is explained in the [CLI README](../crates/textcase-cli/README.md).

Two commands answer most questions interactively:

```bash
textcase lexicon list-sources          # the catalog with license classes
textcase lexicon show-license <source> # per-source licensing and setup guidance
```

## 1. Choose a source

| source | class | what it gives you | best first use | fetch mode | prepare kinds | acknowledgement |
| --- | --- | --- | --- | --- | --- | --- |
| `wikidata` | green | multilingual entity labels and aliases | product names, organizations, famous people, city names | URL-driven | `canonical-map`, `multiword-map` | none |
| `gnd` | green | authority names for persons, places, works, and institutions | German and European proper nouns | URL-driven | `canonical-map`, `multiword-map`, `protected-forms` | none |
| `orcid` | green | researcher names and affiliations | academic corpora and author lists | URL-driven | `canonical-map`, `multiword-map` | none |
| `musicbrainz` | green | artists, bands, labels, releases, works | music and media titles | URL-driven | `canonical-map`, `multiword-map`, `protected-forms` | none |
| `geonames` | yellow | country- and world-scale gazetteer names | geographic proper nouns | built-in | `canonical-map`, `multiword-map` | none |
| `getty` | yellow | art, heritage, and museum vocabulary | museums, artworks, styles, places of culture | URL-driven | `canonical-map`, `multiword-map` | none |
| `wiktionary` | orange | lexical hints, inflected forms, alternate spellings | optional lexical enrichment and German/common-word recovery | built-in | `word-set`, `ranked-candidates` | `--acknowledge-share-alike` |
| `dbpedia` | orange | broad entity labels from DBpedia | broad entity recovery when share-alike is acceptable | URL-driven | `canonical-map`, `multiword-map` | `--acknowledge-share-alike` |
| `openstreetmap` | orange | locality, street, and venue names | street-level and locality-heavy text | URL-driven | `canonical-map`, `multiword-map` | `--acknowledge-odbl` |
| `ud-german-gsd` | orange | ranked candidate hints from real German syntax data | German aggressive mode | built-in | `ranked-candidates` | `--acknowledge-cc-by-sa` |

Starter stacks for common needs:

- **Clean proper nouns** — `wikidata`, `gnd`, `orcid`, and `musicbrainz`: the cleanest redistribution story with the highest-value coverage.
- **Place names** — `geonames` first for city, region, and country names; add `openstreetmap` only when you need street-level or POI coverage and can absorb ODbL obligations.
- **German enrichment** — `gnd` for proper nouns, `wiktionary` for lexical hints, `ud-german-gsd` for ranked candidates in aggressive German mode.

## 2. Check the license

Classes describe the redistribution story, enforced by the CLI:

- `green`: clean redistribution; safest default for redistribution-heavy products.
- `yellow`: practical, but plan for attribution-aware deployment.
- `orange`: stronger obligations (share-alike, ODbL, CC BY-SA); strictly opt-in. `fetch` and `prepare` refuse to run without the source's acknowledgement flag, and the obligation is recorded in the plugin metadata.

The reasoning behind the classes is in [licensing-policy.md](licensing-policy.md); `show-license <source>` prints the specifics.

## 3. Fetch and prepare

There are two fetch styles:

- **Built-in workflows** (`geonames`, `wiktionary`, `ud-german-gsd`): the upstream is a stable, dataset-like download, so the CLI knows how to get it.
- **URL-driven workflows** (everything else): the upstream is a query-oriented API and the right corpus depends on your project, so you pass the exact slice with `--url`. The CLI still validates the payload and records provenance in the raw sidecar manifest.

Each source below follows the same template: what it provides, when to use it, and the commands.

### `geonames` (built-in, yellow)

Canonical place names, alternate names, and broad geographic coverage with stable country-scoped downloads. The strongest general-purpose place-name source.

```bash
textcase lexicon fetch geonames --country DE --output-dir data/raw   # omit --country for the global dump
textcase lexicon prepare geonames --input data/raw/geonames-de.tsv \
    --output data/prepared/geonames-de.json --kind canonical-map --lang de
```

### `wiktionary` (built-in, orange)

Language-specific surface forms from Kaikki/Wiktextract-backed Wiktionary exports: lexical hints, inflected forms, alternate spellings. An opt-in lexical source, not a default proper-noun authority. Built-in editions: `de`, `es`, `fr`, `it`, `nl`, `pl`, `pt`, `tr`, `cs`, and `en`.

```bash
textcase lexicon fetch wiktionary --lang de --acknowledge-share-alike --output-dir data/raw
textcase lexicon prepare wiktionary --input data/raw/wiktionary-de.jsonl.gz \
    --output data/prepared/wiktionary-de.json --kind word-set --lang de --acknowledge-share-alike
```

Prepare with `--kind ranked-candidates` instead when you want candidate scoring rather than a plain lexical set.

### `ud-german-gsd` (built-in, orange)

Noun and proper-noun candidate scoring derived from the UD German GSD treebank. Exists for one purpose: feeding German aggressive mode with ranked candidates (see [german.md](german.md)).

```bash
textcase lexicon fetch ud-german-gsd --acknowledge-cc-by-sa --output-dir data/raw
textcase lexicon prepare ud-german-gsd --input data/raw/ud-german-gsd-r2.13.conllu \
    --output data/prepared/ud-german-gsd.json --kind ranked-candidates --lang de --acknowledge-cc-by-sa
```

### `wikidata` (URL-driven, green)

Entity labels and aliases — people, organizations, places, products, notable works — with a clean CC0 story. The broadest multilingual coverage of any source here. Use `Special:EntityData/<QID>.json` exports, or generate a curated JSON file containing an `entities` map.

```bash
textcase lexicon fetch wikidata --lang en \
    --url "https://www.wikidata.org/wiki/Special:EntityData/Q64.json" --output-dir data/raw
```

### `gnd` (URL-driven, green)

Preferred names, variant names, and structured person/place/work authority records. Authority-grade quality, especially valuable for German and European corpora. Use the Lobid GND API; single records and search result feeds both work.

```bash
textcase lexicon fetch gnd --lang de \
    --url "https://lobid.org/gnd/search?q=Goethe&format=json" --output-dir data/raw
```

### `orcid` (URL-driven, green)

Researcher names and affiliation-style person metadata from the public ORCID API. Excellent for curated academic name lists. Use the personal-details endpoints for the researcher set you care about.

```bash
textcase lexicon fetch orcid --lang en \
    --url "https://pub.orcid.org/v3.0/0000-0002-1825-0097/personal-details" --output-dir data/raw
```

### `musicbrainz` (URL-driven, green)

Artist, release, work, and recording names from MusicBrainz search results or single entities. The strongest source in this catalog for music, artists, labels, and release titles. Use the `ws/2` endpoints with `fmt=json`.

```bash
textcase lexicon fetch musicbrainz --lang en \
    --url "https://musicbrainz.org/ws/2/artist?query=artist:Kraftwerk&fmt=json" --output-dir data/raw
```

### `getty` (URL-driven, yellow)

Labels and identifying strings from Getty linked-art records. Ideal for museums, heritage terms, art styles, and cultural place names. Open an AAT, TGN, or other Getty vocabulary record and use its linked-art JSON representation.

```bash
textcase lexicon fetch getty --lang en \
    --url "https://vocab.getty.edu/aat/300033618.json" --output-dir data/raw
```

### `dbpedia` (URL-driven, orange)

Labels and redirects from DBpedia lookup results or resource graphs. Optional broad entity coverage when share-alike obligations are acceptable. Use the DBpedia Lookup API or a resource JSON response.

```bash
textcase lexicon fetch dbpedia --lang en --acknowledge-share-alike \
    --url "https://lookup.dbpedia.org/api/search?query=Berlin&format=json" --output-dir data/raw
```

### `openstreetmap` (URL-driven, orange)

Place, street, and namedetail strings from Nominatim JSON results. Best when you need street-level or hyperlocal names GeoNames does not cover. Use a Nominatim search URL scoped to the area and result type you actually need.

```bash
textcase lexicon fetch openstreetmap --region DE --acknowledge-odbl \
    --url "https://nominatim.openstreetmap.org/search?format=jsonv2&q=Berlin&namedetails=1" --output-dir data/raw
```

## 4. Pick the prepare kind

`prepare --kind` decides what the plugin can do at runtime:

- `canonical-map` — exact restoration of known proper nouns (`github` → `GitHub`).
- `multiword-map` — the same for multi-token names (`new york` → `New York`); use when your source is rich in them.
- `protected-forms` — exact-casing preservation for sources like `gnd` and `musicbrainz` where the recorded form is authoritative.
- `word-set` — plain "known word" membership checks for lexical sources.
- `ranked-candidates` — lowercase input maps to scored case candidates; consumed by German aggressive mode.

The container formats (`--format json|fst`) are documented in [plugin-format.md](plugin-format.md).
