mod support;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use textcase::LexiconProvider;

fn lookup_benches(c: &mut Criterion) {
    let json = support::json_plugin_set(1024);
    let fst = support::fst_plugin_set(1024);

    let mut group = c.benchmark_group("lookup");
    group.bench_function("json-canonical-form", |b| {
        b.iter(|| black_box(json.canonical_form("en", "entry-00512")))
    });
    group.bench_function("fst-canonical-form", |b| {
        b.iter(|| black_box(fst.canonical_form("en", "entry-00512")))
    });
    group.bench_function("json-membership", |b| {
        b.iter(|| black_box(json.contains_word("en", "entry-00512")))
    });
    group.bench_function("fst-membership", |b| {
        b.iter(|| black_box(fst.contains_word("en", "entry-00512")))
    });
    group.finish();
}

criterion_group!(benches, lookup_benches);
criterion_main!(benches);
