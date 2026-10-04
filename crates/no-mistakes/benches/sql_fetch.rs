use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn parse_sql_without_fetch(c: &mut Criterion) {
    let sql = "SELECT id, payload FROM events WHERE tenant_id = $1 AND created_at >= $2 ORDER BY created_at DESC, id DESC LIMIT 50";
    c.bench_function("sql_parse_without_fetch", |b| {
        b.iter(|| {
            black_box(no_mistakes::codebase::postgres::parse_postgres_sql(
                black_box(sql),
            ))
        })
    });
}

criterion_group!(benches, parse_sql_without_fetch);
criterion_main!(benches);
