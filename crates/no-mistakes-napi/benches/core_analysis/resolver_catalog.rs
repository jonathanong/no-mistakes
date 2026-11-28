use super::shard;
use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use no_mistakes::codebase::ts_resolver::TsConfigCatalog;
use no_mistakes::codebase::ts_source::discover_visible_paths;
use std::path::PathBuf;

pub(super) fn bench_resolver_catalog(c: &mut Criterion) {
    if !shard::should_run(shard::QUERY) {
        return;
    }
    let root = std::env::var_os("NO_MISTAKES_CATALOG_BENCH_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .canonicalize()
        .expect("resolver catalog corpus exists");
    // Discovery belongs to the request boundary, outside the measured constructor.
    let visible = discover_visible_paths(&root);
    assert!(visible.len() > 1_000);
    let roots = [root.clone()];
    let mut group = c.benchmark_group("resolver_catalog_visible");
    group.throughput(Throughput::Elements(visible.len() as u64));
    for threads in [1, 4] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(threads), &threads, |b, _| {
            b.iter(|| {
                pool.install(|| {
                    black_box(TsConfigCatalog::from_visible(
                        black_box(&root),
                        black_box(&roots),
                        black_box(&visible),
                    ))
                })
            });
        });
    }
    group.finish();
}
