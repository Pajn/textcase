# Source setup guide

`textcase` keeps source setup explicit on purpose: every plugin should have a clear origin, a clear license story, and a workflow that a new user can follow without reading the implementation.

There are two fetch styles:

- **Built-in workflows** for sources with stable, dataset-like upstream downloads.
- **URL-driven workflows** for query-oriented APIs where the right corpus depends on your project.

Run `textcase lexicon list-sources` to see the full catalog, then `textcase lexicon show-license <source>` for the exact acknowledgement and setup guidance for one source.

## Which source should I pick?

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

## Recommended starter stacks

### Clean proper nouns

Use `wikidata`, `gnd`, `orcid`, and `musicbrainz` when you want the cleanest redistribution story and the highest-value proper-noun coverage.

### Place names

Use `geonames` first for city, region, and country names. Add `openstreetmap` only when you need street-level or local POI coverage and you are prepared for ODbL obligations.

### German enrichment

Use `gnd` for proper nouns, `wiktionary` for lexical hints, and `ud-german-gsd` for ranked candidates in aggressive German mode.

## Built-in workflows

These sources can be fetched directly without a custom URL.

### `geonames`

**Why use it:** strong general-purpose place-name coverage with stable, country-scoped downloads.

**What it provides:** canonical place names, alternate names, and broad geographic coverage.

**Fetch:**

```bash
textcase lexicon fetch geonames --country DE --output-dir data/raw
```

Omit `--country` to fetch the global `allCountries` dump.

**Prepare:**

```bash
textcase lexicon prepare geonames --input data/raw/geonames-de.tsv --output data/prepared/geonames-de.json --kind canonical-map --lang de
```

### `wiktionary`

**Why use it:** useful when you need lexical hints, inflected forms, or optional candidate enrichment beyond proper nouns.

**What it provides:** language-specific surface forms from Kaikki/Wiktextract-backed Wiktionary exports. `textcase` treats Wiktionary as an opt-in lexical source, not a default proper-noun authority.

**Built-in editions:** `de`, `es`, `fr`, `it`, `nl`, `pl`, `pt`, `tr`, `cs`, and `en`.

**Fetch:**

```bash
textcase lexicon fetch wiktionary --lang de --acknowledge-share-alike --output-dir data/raw
```

**Prepare:**

```bash
textcase lexicon prepare wiktionary --input data/raw/wiktionary-de.jsonl.gz --output data/prepared/wiktionary-de.json --kind word-set --lang de --acknowledge-share-alike
```

Use `ranked-candidates` when you want candidate scoring rather than a plain lexical set.

### `ud-german-gsd`

**Why use it:** optional ranking data for German aggressive mode.

**What it provides:** noun and proper-noun candidate scoring derived from the UD German GSD treebank.

**Fetch:**

```bash
textcase lexicon fetch ud-german-gsd --acknowledge-cc-by-sa --output-dir data/raw
```

**Prepare:**

```bash
textcase lexicon prepare ud-german-gsd --input data/raw/ud-german-gsd-r2.13.conllu --output data/prepared/ud-german-gsd.json --kind ranked-candidates --lang de --acknowledge-cc-by-sa
```

## URL-driven workflows

These sources are intentionally URL-driven because the right upstream slice depends on your domain. The CLI still validates the fetched payload and records source provenance in the raw sidecar manifest.

### `wikidata`

**Why use it:** broad multilingual proper-noun coverage with a clean CC0 story.

**What it provides:** entity labels and aliases, especially for people, organizations, places, products, and notable works.

**Find a URL:** use Wikidata entity exports such as `Special:EntityData/<QID>.json`, or generate a curated JSON file containing an `entities` map.

**Example:**

```bash
textcase lexicon fetch wikidata --lang en --url "https://www.wikidata.org/wiki/Special:EntityData/Q64.json" --output-dir data/raw
```

### `gnd`

**Why use it:** authority-grade names with especially good value for German and European corpora.

**What it provides:** preferred names, variant names, and structured person/place/work authority records.

**Find a URL:** use the Lobid GND API. Single records and search result feeds are both supported.

**Example:**

```bash
textcase lexicon fetch gnd --lang de --url "https://lobid.org/gnd/search?q=Goethe&format=json" --output-dir data/raw
```

### `orcid`

**Why use it:** excellent for curated academic name lists.

**What it provides:** researcher names and affiliation-style person metadata from the public ORCID API.

**Find a URL:** use ORCID public API personal-details endpoints for the researcher set you care about.

**Example:**

```bash
textcase lexicon fetch orcid --lang en --url "https://pub.orcid.org/v3.0/0000-0002-1825-0097/personal-details" --output-dir data/raw
```

### `musicbrainz`

**Why use it:** the strongest source in this crate for music, artists, labels, and release titles.

**What it provides:** artist, release, work, and recording names from MusicBrainz search results or single entities.

**Find a URL:** use the MusicBrainz `ws/2` endpoints with `fmt=json`.

**Example:**

```bash
textcase lexicon fetch musicbrainz --lang en --url "https://musicbrainz.org/ws/2/artist?query=artist:Kraftwerk&fmt=json" --output-dir data/raw
```

### `getty`

**Why use it:** ideal for museums, heritage terms, art styles, and cultural place names.

**What it provides:** labels and identifying strings from Getty linked-art records.

**Find a URL:** open an AAT, TGN, or other Getty vocabulary record and use its linked-art JSON representation.

**Example:**

```bash
textcase lexicon fetch getty --lang en --url "https://vocab.getty.edu/aat/300033618.json" --output-dir data/raw
```

### `dbpedia`

**Why use it:** optional broad entity coverage when share-alike obligations are acceptable.

**What it provides:** labels and redirects from DBpedia lookup results or resource graphs.

**Find a URL:** use the DBpedia Lookup API or a DBpedia resource JSON response.

**Example:**

```bash
textcase lexicon fetch dbpedia --lang en --acknowledge-share-alike --url "https://lookup.dbpedia.org/api/search?query=Berlin&format=json" --output-dir data/raw
```

### `openstreetmap`

**Why use it:** best when you need street-level or hyperlocal names that GeoNames does not cover.

**What it provides:** place, street, and namedetail strings from Nominatim JSON results.

**Find a URL:** use a Nominatim search URL scoped to the area and result type you actually need.

**Example:**

```bash
textcase lexicon fetch openstreetmap --region DE --acknowledge-odbl --url "https://nominatim.openstreetmap.org/search?format=jsonv2&q=Berlin&namedetails=1" --output-dir data/raw
```

## Picking the right plugin kind

- Use `canonical-map` for exact restoration of known proper nouns.
- Use `multiword-map` when your source contains many multi-token names.
- Use `protected-forms` for sources like `gnd` and `musicbrainz` where preserving exact casing matters.
- Use `word-set` for lexical “known word” membership checks.
- Use `ranked-candidates` when you want lowercase input to map to scored case candidates.

## Licensing summary

- `green` sources are the safest defaults for redistribution-heavy products.
- `yellow` sources are practical but need attribution-aware deployment.
- `orange` sources are strictly opt-in and should be enabled only when your deployment and licensing model can absorb their obligations.
