use super::shard;
use criterion::{black_box, Criterion, Throughput};
use no_mistakes::benchmark_support;
use std::collections::HashMap;
use std::path::PathBuf;

pub(super) fn bench_symbol_index_build_and_lookup(c: &mut Criterion) {
    if !shard::should_run(shard::QUERY) {
        return;
    }
    const IMPORTERS: usize = 1_024;
    const SYMBOLS_PER_IMPORTER: usize = 8;

    let source = PathBuf::from("/benchmark/src/source.ts");
    let imports = (0..IMPORTERS)
        .map(|importer| {
            (
                PathBuf::from(format!("/benchmark/src/importer-{importer}.ts")),
                (0..SYMBOLS_PER_IMPORTER)
                    .map(|symbol| {
                        (
                            source.clone(),
                            format!("symbol-{symbol}"),
                            format!("local-{symbol}"),
                            symbol % 3 == 0,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<HashMap<_, _>>();
    let index = no_mistakes::codebase::dependencies::graph::SymbolIndex::build(&imports);
    assert_eq!(index.file_importers(&source).len(), IMPORTERS);
    assert_eq!(
        index.importers_of(&source, "symbol-0").unwrap().len(),
        IMPORTERS
    );

    let mut group = c.benchmark_group("symbol_index");
    group.throughput(Throughput::Elements(
        (IMPORTERS * SYMBOLS_PER_IMPORTER) as u64,
    ));
    group.bench_function("build", |b| {
        b.iter(|| {
            black_box(
                no_mistakes::codebase::dependencies::graph::SymbolIndex::build(black_box(&imports)),
            )
        });
    });
    group.bench_function("lookup", |b| {
        b.iter(|| {
            let symbol_importers = index
                .importers_of(black_box(&source), black_box("symbol-0"))
                .unwrap();
            let file_importers = index.file_importers(black_box(&source));
            black_box((symbol_importers.len(), file_importers.len()))
        });
    });
    let missing_source = PathBuf::from("/benchmark/src/missing.ts");
    assert!(index.importers_of(&source, "missing-symbol").is_none());
    assert!(index.importers_of(&missing_source, "symbol-0").is_none());
    // Isolate borrowed table probes from the owned file-importer projection.
    group.throughput(Throughput::Elements(1));
    for (label, path, symbol) in [
        ("lookup_hit", source.as_path(), "symbol-0"),
        ("lookup_missing_symbol", source.as_path(), "missing-symbol"),
        (
            "lookup_missing_source",
            missing_source.as_path(),
            "symbol-0",
        ),
    ] {
        group.bench_function(label, |b| {
            b.iter(|| black_box(index.importers_of(black_box(path), black_box(symbol))));
        });
    }
    group.finish();
}

pub(super) fn bench_symbol_index_distinct_target_build(c: &mut Criterion) {
    if !shard::should_run(shard::QUERY) {
        return;
    }
    const DISTINCT_KEYS: usize = 2_048;
    let mut group = c.benchmark_group("symbol_index");
    group.throughput(Throughput::Elements(DISTINCT_KEYS as u64));
    // Keep both many-source and one-source/many-symbol construction controls.
    for distinct_sources in [true, false] {
        let source = PathBuf::from("/benchmark/src/source.ts");
        let entries = (0..DISTINCT_KEYS)
            .map(|key| {
                (
                    if distinct_sources {
                        PathBuf::from(format!("/benchmark/src/target-{key}.ts"))
                    } else {
                        source.clone()
                    },
                    format!("symbol-{key}"),
                    format!("local-{key}"),
                    false,
                )
            })
            .collect::<Vec<_>>();
        let imports = HashMap::from([(PathBuf::from("/benchmark/src/importer.ts"), entries)]);
        let index = no_mistakes::codebase::dependencies::graph::SymbolIndex::build(&imports);
        let first_source = if distinct_sources {
            PathBuf::from("/benchmark/src/target-0.ts")
        } else {
            source
        };
        assert_eq!(index.file_importers(&first_source).len(), 1);
        assert_eq!(
            index.importers_of(&first_source, "symbol-0").unwrap().len(),
            1
        );
        let label = if distinct_sources {
            "distinct_target_build"
        } else {
            "distinct_symbol_build"
        };
        group.bench_function(label, |b| {
            b.iter(|| {
                black_box(
                    no_mistakes::codebase::dependencies::graph::SymbolIndex::build(black_box(
                        &imports,
                    )),
                )
            });
        });
    }
    group.finish();
}

pub(super) fn bench_scoped_resolver_selection(c: &mut Criterion) {
    if !shard::should_run(shard::QUERY) {
        return;
    }
    const REQUESTS: usize = 1_024;
    let fixture = benchmark_support::scoped_resolver_selection_fixture();
    assert_eq!(
        benchmark_support::resolve_repeated_scoped_imports(&fixture, REQUESTS),
        benchmark_support::ScopedResolverSelectionSummary {
            resolved: REQUESTS,
            selection_builds: 1,
        }
    );

    let mut group = c.benchmark_group("scoped_resolver_request");
    group.throughput(Throughput::Elements(REQUESTS as u64));
    group.bench_function("repeated_imports_from_one_file", |b| {
        b.iter(|| {
            black_box(benchmark_support::resolve_repeated_scoped_imports(
                black_box(&fixture),
                REQUESTS,
            ))
        });
    });
    const RESOLVERS: usize = 128;
    assert_eq!(
        benchmark_support::build_repeated_scoped_resolvers(&fixture, RESOLVERS),
        RESOLVERS
    );
    group.throughput(Throughput::Elements(RESOLVERS as u64));
    group.bench_function("repeated_construction", |b| {
        b.iter(|| {
            black_box(benchmark_support::build_repeated_scoped_resolvers(
                black_box(&fixture),
                RESOLVERS,
            ))
        });
    });
    group.finish();
}
