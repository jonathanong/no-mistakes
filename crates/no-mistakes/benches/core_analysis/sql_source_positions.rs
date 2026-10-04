use criterion::{black_box, Criterion};
use no_mistakes::codebase::postgres::statements::extract_sql_statement_facts;

pub(super) fn bench_sql_source_positions(c: &mut Criterion) {
    for (name, sql) in [
        (
            "insert_ranges_512",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../test-cases/postgres/source-positions/fixture/many-inserts.sql"
            )),
        ),
        (
            "offset_positions_512",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../test-cases/postgres/source-positions/fixture/many-offsets.sql"
            )),
        ),
    ] {
        c.bench_function(name, |b| {
            b.iter(|| black_box(extract_sql_statement_facts(black_box(sql))))
        });
    }
}
