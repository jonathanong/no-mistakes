// Reproduction asset for lateral-scope-snapshots.md.
use criterion::{black_box, Criterion};
use no_mistakes::codebase::postgres::statements::extract_sql_statement_facts;

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture/sql");
    let mut criterion = Criterion::default()
        .sample_size(30)
        .warm_up_time(std::time::Duration::from_secs(2))
        .measurement_time(std::time::Duration::from_secs(4))
        .configure_from_args();
    for size in [16, 32, 64, 128] {
        let sql =
            std::fs::read_to_string(root.join(format!("lateral-scope-growth-{size}.sql"))).unwrap();
        let expected = extract_sql_statement_facts(&sql);
        assert!(!expected.parse_failed);
        assert_eq!(expected.deletes.len(), 1);
        criterion.bench_function(&format!("lateral_scope_{size}"), |bench| {
            bench.iter(|| black_box(extract_sql_statement_facts(black_box(&sql))))
        });
    }
    criterion.final_summary();
}
