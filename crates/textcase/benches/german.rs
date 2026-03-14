mod support;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use textcase::{GermanMode, convert};

fn german_benches(c: &mut Criterion) {
    let input = "ich mag die wissenschaft und die analyse in berlin";
    let ranked = support::german_ranked_plugin_set(256);
    let conservative = support::german_options(GermanMode::Conservative, None);
    let balanced = support::german_options(GermanMode::Balanced, None);
    let aggressive = support::german_options(GermanMode::Aggressive, Some(&ranked));

    let mut group = c.benchmark_group("german");
    group.bench_function("conservative", |b| {
        b.iter(|| convert(black_box(input), black_box(&conservative)))
    });
    group.bench_function("balanced", |b| {
        b.iter(|| convert(black_box(input), black_box(&balanced)))
    });
    group.bench_function("aggressive-with-ranked", |b| {
        b.iter(|| convert(black_box(input), black_box(&aggressive)))
    });
    group.finish();
}

criterion_group!(benches, german_benches);
criterion_main!(benches);
