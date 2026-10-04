#[path = "core_analysis/sql_source_positions.rs"]
mod sql_source_positions;
use criterion::{criterion_group, criterion_main};
use sql_source_positions::bench_sql_source_positions;
criterion_group!(benches, bench_sql_source_positions);
criterion_main!(benches);
