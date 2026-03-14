mod support;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use textcase::convert;

fn casing_benches(c: &mut Criterion) {
    let short = support::sample_text();
    let long = support::long_text();
    let sentence = support::sentence_options();
    let title = support::title_options();

    let mut group = c.benchmark_group("casing");
    group.bench_function("sentence-short", |b| {
        b.iter(|| convert(black_box(short), black_box(&sentence)))
    });
    group.bench_function("sentence-long", |b| {
        b.iter(|| convert(black_box(&long), black_box(&sentence)))
    });
    group.bench_function("sentence-title", |b| {
        b.iter(|| convert(black_box(short), black_box(&title)))
    });
    group.finish();
}

criterion_group!(benches, casing_benches);
criterion_main!(benches);
