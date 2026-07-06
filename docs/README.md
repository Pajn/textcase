# Documentation

Topic guides for the `textcase` workspace. If you are new, don't start here — start with the README that matches what you're doing:

- **Using the library** → [crates/textcase/README.md](../crates/textcase/README.md): modes, options, behavior, languages, loading plugins.
- **Building lexicon plugins** → [crates/textcase-cli/README.md](../crates/textcase-cli/README.md): install, the fetch → prepare → build-plugin pipeline, a worked example.

The guides in this folder go deeper on one topic each:

| Guide | Read it when |
| --- | --- |
| [sources.md](sources.md) | you are picking a data source for a plugin, or need the fetch/prepare commands for a specific one |
| [german.md](german.md) | you want German noun capitalization recovered and need to choose a `GermanMode` tier |
| [plugin-format.md](plugin-format.md) | you need to read or emit the JSON/FST plugin containers without the CLI |
| [licensing-policy.md](licensing-policy.md) | you need to know why sources are classed green/yellow/orange and what that means for redistribution |
| [architecture.md](architecture.md) | you are contributing and want the crate/module layout |
| [performance.md](performance.md) | you want to run the benchmarks or see the current numbers |
