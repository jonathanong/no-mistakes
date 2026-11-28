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
    bench_scoped_callable_indexes(c);
}

fn bench_scoped_callable_indexes(c: &mut Criterion) {
    use no_mistakes::benchmark_support::{
        construct_scoped_callable_index, prepare_scoped_callable_index, scoped_callable_fixture,
    };
    let mut group = c.benchmark_group("scoped_callable_index");
    for (label, sparse) in [("dense", false), ("sparse", true)] {
        let fixture = scoped_callable_fixture(4_096, sparse);
        assert!(construct_scoped_callable_index(&fixture) >= 4_096);
        let index = prepare_scoped_callable_index(&fixture);
        assert_eq!(index.local_id(fixture.scope, &fixture.binding), Some(0));
        assert_eq!(
            index.local_id(fixture.deep_scope, &fixture.binding),
            Some(0)
        );
        assert!(index.local_id(fixture.deep_scope, "missing").is_none());
        assert!(index.alias_resolves(fixture.scope, &fixture.alias));
        assert!(index.binding_live(fixture.scope, &fixture.binding));
        assert_eq!(
            index.class_id(fixture.class_scope, "Class0.m0"),
            Some(2_000_000)
        );
        group.bench_function(BenchmarkId::new(label, "construct"), |b| {
            b.iter(|| black_box(construct_scoped_callable_index(black_box(&fixture))));
        });
        for (probe, scope, name) in [
            ("local_hit", fixture.scope, fixture.binding.as_str()),
            ("local_miss", fixture.scope, "missing"),
            ("deep_hit", fixture.deep_scope, fixture.binding.as_str()),
            ("deep_miss", fixture.deep_scope, "missing"),
        ] {
            group.bench_function(BenchmarkId::new(label, probe), |b| {
                b.iter(|| black_box(index.local_id(black_box(scope), black_box(name))));
            });
        }
        group.bench_function(BenchmarkId::new(label, "alias_hit"), |b| {
            b.iter(|| {
                black_box(index.alias_resolves(black_box(fixture.scope), black_box(&fixture.alias)))
            });
        });
        group.bench_function(BenchmarkId::new(label, "binding_live"), |b| {
            b.iter(|| {
                black_box(index.binding_live(black_box(fixture.scope), black_box(&fixture.binding)))
            });
        });
        group.bench_function(BenchmarkId::new(label, "class_hit"), |b| {
            b.iter(|| {
                black_box(index.class_id(black_box(fixture.class_scope), black_box("Class0.m0")))
            });
        });
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
