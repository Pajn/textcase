# German support

German support is intentionally tiered.

## Conservative

- sentence-start capitalization
- subtitle-start capitalization
- acronym and mixed-case preservation
- lexicon-assisted proper noun recovery

This mode is safe by default but does not try to recover ordinary noun capitalization from lowercase text.

## Balanced

Balanced mode adds hand-authored heuristics:

- article + noun-like token
- preposition + article + noun-like token
- common noun suffix hints (`-ung`, `-heit`, `-keit`, `-schaft`, `-tion`, ...)
- small ambiguity list to avoid obvious false positives

## Aggressive

Aggressive mode keeps balanced heuristics and additionally uses ranked-candidate plugin data when available.

This improves recall when a prepared `ranked-candidates` plugin exists, especially for optional workflows such as `ud-german-gsd`.
