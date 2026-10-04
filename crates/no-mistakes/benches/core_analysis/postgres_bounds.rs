use criterion::{black_box, BenchmarkId, Criterion};
use no_mistakes::benchmark_support::evaluate_postgres_bounds;
use no_mistakes::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};
use no_mistakes::codebase::ts_source::{FileInventory, SourceStore};
use std::{path::PathBuf, sync::Arc};

pub(super) fn bench_postgres_bounds(c: &mut Criterion) {
    if !super::shard::should_run(super::shard::QUERY) {
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/postgres-bounds");
    let mut group = c.benchmark_group("postgres_bounds");
    for size in [16, 64, 256] {
        let catalog_name = format!("catalog-{size}.json");
        let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[
            root.join(&catalog_name)
        ])));
        let catalog = SchemaCatalog::load(&root, &catalog_name, &sources).unwrap();
        group.bench_function(BenchmarkId::new("catalog_load", size), |b| {
            b.iter(|| {
                black_box(
                    SchemaCatalog::load(
                        black_box(&root),
                        black_box(&catalog_name),
                        black_box(&sources),
                    )
                    .unwrap(),
                )
            })
        });
        for kind in ["reversed", "cycle"] {
            let sql = std::fs::read_to_string(root.join(format!("{kind}-{size}.sql"))).unwrap();
            let facts = extract_sql_statement_facts(&sql);
            assert_eq!(
                evaluate_postgres_bounds(&facts, &catalog),
                if kind == "cycle" { size } else { 0 }
            );
            group.bench_with_input(BenchmarkId::new(kind, size), &facts, |b, facts| {
                b.iter(|| {
                    black_box(evaluate_postgres_bounds(
                        black_box(facts),
                        black_box(&catalog),
                    ))
                })
            });
        }
        group.bench_function(BenchmarkId::new("catalog_fallback", size), |b| {
            b.iter(|| black_box(catalog.relation(black_box("t0"))))
        });
    }
    group.finish();
}
