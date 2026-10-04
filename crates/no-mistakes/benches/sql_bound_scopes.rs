#[path = "core_analysis/postgres_scopes.rs"]
mod postgres_scopes;
use criterion::{criterion_group, criterion_main};
use postgres_scopes::bench_scopes;
criterion_group!(benches, bench_scopes);
criterion_main!(benches);
