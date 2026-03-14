mod support;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use textcase::PluginSet;

fn plugin_loading_benches(c: &mut Criterion) {
    let small_json = support::json_plugin_bytes(128);
    let medium_json = support::json_plugin_bytes(1024);
    let large_json = support::json_plugin_bytes(8192);
    let small_fst = support::fst_plugin_path(128);
    let medium_fst = support::fst_plugin_path(1024);
    let large_fst = support::fst_plugin_path(8192);

    let mut group = c.benchmark_group("plugin-loading");
    group.bench_function("json-small", |b| {
        b.iter(|| PluginSet::from_json_bytes(black_box(&small_json)).expect("json small"))
    });
    group.bench_function("json-medium", |b| {
        b.iter(|| PluginSet::from_json_bytes(black_box(&medium_json)).expect("json medium"))
    });
    group.bench_function("json-large", |b| {
        b.iter(|| PluginSet::from_json_bytes(black_box(&large_json)).expect("json large"))
    });
    group.bench_function("fst-small", |b| {
        b.iter(|| PluginSet::from_fst_path(black_box(&small_fst)).expect("fst small"))
    });
    group.bench_function("fst-medium", |b| {
        b.iter(|| PluginSet::from_fst_path(black_box(&medium_fst)).expect("fst medium"))
    });
    group.bench_function("fst-large", |b| {
        b.iter(|| PluginSet::from_fst_path(black_box(&large_fst)).expect("fst large"))
    });
    group.finish();
}

criterion_group!(benches, plugin_loading_benches);
criterion_main!(benches);
