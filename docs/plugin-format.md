# Plugin format

`textcase` supports two plugin containers.

## JSON plugin

JSON plugins are fully inspectable and serialize a `PluginSchema` value containing:

- schema version
- plugin name
- plugin kind
- locale list
- license summary
- source metadata
- generated timestamp
- checksum
- payload

Supported payload kinds:

- `word-set`
- `canonical-map`
- `multiword-map`
- `ranked-candidates`
- `protected-forms`

## FST plugin

FST plugins store the lookup payload in an `.tclx` file and metadata/value tables in a sidecar `<name>.tclx.meta.json` file.

This keeps runtime lookup compact while preserving inspectable metadata and deterministic rebuilds.
