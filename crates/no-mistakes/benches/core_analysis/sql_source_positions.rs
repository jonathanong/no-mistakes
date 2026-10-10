use criterion::{black_box, Criterion};
use no_mistakes::codebase::postgres::statements::extract_sql_statement_facts;
use no_mistakes::codebase::postgres::{parse_postgres_source, PostgresSqlSource};

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
    for (name, sql) in [
        (
            "procedural_label_mismatches_32",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../fixtures/postgres-facts/source/procedural-label-mismatches-32.sql"
            )),
        ),
        (
            "procedural_label_mismatches_256",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../fixtures/postgres-facts/source/procedural-label-mismatches-256.sql"
            )),
        ),
    ] {
        let source = PostgresSqlSource {
            sql: sql.to_owned(),
            file_name: None,
        };
        c.bench_function(name, |b| {
            b.iter(|| black_box(parse_postgres_source(black_box(&source))))
        });
    }
}
