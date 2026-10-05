use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use no_mistakes::codebase::postgres::extract_sql_statement_facts;

pub(super) fn bench_scopes(c: &mut Criterion) {
    let mut group = c.benchmark_group("postgres_independent_ctes");
    for (count, sql) in [
        (
            32,
            include_str!("../../../../fixtures/performance/postgres-scopes/independent-32.sql"),
        ),
        (
            128,
            include_str!("../../../../fixtures/performance/postgres-scopes/independent-128.sql"),
        ),
        (
            512,
            include_str!("../../../../fixtures/performance/postgres-scopes/independent-512.sql"),
        ),
    ] {
        let facts = extract_sql_statement_facts(sql);
        assert!(!facts.parse_failed);
        assert_eq!(facts.bounds.len(), 1);
        group.throughput(Throughput::Elements(count));
        group.bench_with_input(BenchmarkId::from_parameter(count), &sql, |b, sql| {
            b.iter(|| black_box(extract_sql_statement_facts(black_box(sql))));
        });
    }
    group.finish();
}
