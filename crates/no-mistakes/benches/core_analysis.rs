#[path = "core_analysis/aggregate.rs"]
mod aggregate;
#[path = "core_analysis/fixtures.rs"]
mod fixtures;
#[path = "core_analysis/graph.rs"]
mod graph;
#[path = "core_analysis/observer.rs"]
mod observer;
#[path = "core_analysis/reports.rs"]
mod reports;

use aggregate::{bench_aggregate_and_multi_report, bench_impacted_checks};
use criterion::{criterion_group, criterion_main};
use graph::{bench_facts_graph_and_query, bench_lazy_traversal};
use observer::bench_observer_overhead;
use reports::{bench_symbols, bench_workspace};

criterion_group!(
    benches,
    bench_lazy_traversal,
    bench_facts_graph_and_query,
    bench_symbols,
    bench_workspace,
    bench_aggregate_and_multi_report,
    bench_impacted_checks,
    bench_observer_overhead,
);
criterion_main!(benches);
