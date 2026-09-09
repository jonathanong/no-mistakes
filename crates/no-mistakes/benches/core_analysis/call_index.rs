use super::shard;
use criterion::{black_box, BenchmarkId, Criterion, Throughput};

pub(super) fn bench_callable_file_index_construction(c: &mut Criterion) {
    if !shard::should_run(shard::GRAPH_CORE) {
        return;
    }
    let mut group = c.benchmark_group("callable_file_index_construction");
    for entries in [4_096usize, 16_384] {
        let fixture = no_mistakes::benchmark_support::callable_file_index_fixture(entries);
        let summary = no_mistakes::benchmark_support::construct_callable_file_index(&fixture);
        assert!(
            summary.entries >= entries,
            "callable index must retain the synthetic bindings"
        );
        group.throughput(Throughput::Elements(entries as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(entries),
            &fixture,
            |b, fixture| {
                b.iter(|| {
                    black_box(
                        no_mistakes::benchmark_support::construct_callable_file_index(black_box(
                            fixture,
                        )),
                    )
                });
            },
        );
    }
    group.finish();
}

pub(super) fn bench_call_site_membership(c: &mut Criterion) {
    if !shard::should_run(shard::GRAPH_CORE) {
        return;
    }
    let mut group = c.benchmark_group("call_site_membership");
    for files in [4_096usize, 16_384] {
        let hits = no_mistakes::benchmark_support::probe_call_site_files(files);
        assert_eq!(hits, files, "every synthetic file must have a call site");
        group.throughput(Throughput::Elements(files as u64));
        group.bench_with_input(BenchmarkId::from_parameter(files), &files, |b, files| {
            b.iter(|| {
                black_box(no_mistakes::benchmark_support::probe_call_site_files(
                    black_box(*files),
                ))
            });
        });
    }
    group.finish();
}
