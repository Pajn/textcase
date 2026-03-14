# Sources

| source | class | primary use | plugin kinds | acknowledgement |
| --- | --- | --- | --- | --- |
| `wikidata` | green | multilingual proper nouns | canonical-map, multiword-map | none |
| `gnd` | green | authority names | canonical-map, multiword-map, protected-forms | none |
| `orcid` | green | researcher names | canonical-map, multiword-map | none |
| `musicbrainz` | green | artists and works | canonical-map, multiword-map, protected-forms | none |
| `geonames` | yellow | place names | canonical-map, multiword-map | none |
| `getty` | yellow | culture/heritage vocabularies | canonical-map, multiword-map | none |
| `wiktionary` | orange | lexical hints | word-set, ranked-candidates | `--acknowledge-share-alike` |
| `dbpedia` | orange | entity labels | canonical-map, multiword-map | `--acknowledge-share-alike` |
| `openstreetmap` | orange | local place names | canonical-map, multiword-map | `--acknowledge-odbl` |
| `ud-german-gsd` | orange | German ranked candidates | ranked-candidates | `--acknowledge-cc-by-sa` |
| `omw` | gray | lexical hints / experiments | word-set, ranked-candidates | none |

## Recommended defaults

Prefer `wikidata`, `gnd`, `orcid`, and `musicbrainz` when you want the cleanest redistribution story.

Use `geonames` and `getty` when attribution-heavy data is worth the extra compliance work.

Treat `wiktionary`, `dbpedia`, `openstreetmap`, and `ud-german-gsd` as opt-in workflows only.
