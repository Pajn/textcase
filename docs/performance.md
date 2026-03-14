# Performance

The workspace keeps two plugin representations:

- JSON for inspectability and human review
- FST for compact lookup-heavy runtime usage

## Benchmark targets

Runnable Criterion benchmarks live in `crates/textcase/benches/` and are mirrored from the repository-level `benches/` directory.

Run them with:

```bash
cargo bench -p textcase --bench casing
cargo bench -p textcase --bench lookup
cargo bench -p textcase --bench plugin_loading
cargo bench -p textcase --bench german
```

## What to verify

- JSON vs FST lookup behavior for canonical-form and membership queries
- JSON vs FST plugin load costs
- conservative vs balanced vs aggressive German overhead
- casing throughput for short and long inputs

FST plugins are loaded through memory mapping, so the runtime path avoids copying the full binary payload into heap-owned `Vec<u8>` buffers.

## Current benchmark snapshot

The following numbers were captured locally with:

```bash
cargo bench -p textcase --bench lookup -- --sample-size 10
cargo bench -p textcase --bench plugin_loading -- --sample-size 10
cargo bench -p textcase --bench german -- --sample-size 10
cargo bench -p textcase --bench casing -- --sample-size 10
```

Treat them as implementation sanity checks rather than cross-machine guarantees.

### Lookup

- `lookup/json-canonical-form`: about `98-100 ns`
- `lookup/fst-canonical-form`: about `83-84 ns`
- `lookup/json-membership`: about `1.61-1.63 ns`
- `lookup/fst-membership`: about `1.62-1.64 ns`

The mmap-backed FST path is already measurably faster for canonical-form lookup while keeping membership checks in the same range.

### Plugin loading

- `plugin-loading/json-small`: about `88-89 us`
- `plugin-loading/json-medium`: about `838-852 us`
- `plugin-loading/json-large`: about `7.41-7.46 ms`
- `plugin-loading/fst-small`: about `60-65 us`
- `plugin-loading/fst-medium`: about `307-359 us`
- `plugin-loading/fst-large`: about `2.41-2.48 ms`

The FST format scales noticeably better than JSON for load-heavy paths, which matches the design goal for prepared runtime lexicons.

### German heuristics

- `german/conservative`: about `4.88-4.92 us`
- `german/balanced`: about `5.68-5.71 us`
- `german/aggressive-with-ranked`: about `6.49-6.59 us`

Each stronger heuristic mode adds overhead, but the absolute cost remains in low single-digit microseconds for the benchmark fixture.

### Casing throughput

- `casing/sentence-short`: about `5.36-5.42 us`
- `casing/sentence-long`: about `366-371 us`
- `casing/sentence-title`: about `5.40-5.46 us`

These measurements confirm that long-input work grows with text size while short sentence and title recasing stay in the same microsecond band.
