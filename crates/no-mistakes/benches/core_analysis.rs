#[path = "core_analysis/aggregate.rs"]
mod aggregate;
#[path = "core_analysis/fixtures.rs"]
mod fixtures;
#[path = "core_analysis/graph.rs"]
mod graph;
#[path = "core_analysis/graph_gates.rs"]
mod graph_gates;
#[path = "core_analysis/language_frontends.rs"]
mod language_frontends;
#[path = "core_analysis/observer.rs"]
mod observer;
#[path = "core_analysis/query_indexes.rs"]
mod query_indexes;
#[path = "core_analysis/react_traits.rs"]
mod react_traits;
#[path = "core_analysis/relationships.rs"]
mod relationships;
#[path = "core_analysis/reports.rs"]
mod reports;
#[path = "core_analysis/shard.rs"]
mod shard;

use aggregate::{
    bench_aggregate_and_multi_report, bench_finite_set_membership, bench_impacted_checks,
};
use criterion::{criterion_group, criterion_main};
use graph::{bench_facts_graph_and_query, bench_high_fanout_finalization, bench_lazy_traversal};
use graph_gates::bench_graph_gates;
use language_frontends::bench_language_frontends;
use observer::bench_observer_overhead;
use query_indexes::{
    bench_scoped_resolver_selection, bench_symbol_index_build_and_lookup,
    bench_symbol_index_distinct_target_build,
};
use react_traits::bench_react_traits;
use relationships::bench_relationship_projection;
use reports::{bench_symbols, bench_workspace};

criterion_group!(
    benches,
    bench_lazy_traversal,
    bench_facts_graph_and_query,
    bench_graph_gates,
    bench_language_frontends,
    bench_high_fanout_finalization,
    bench_symbol_index_build_and_lookup,
    bench_symbol_index_distinct_target_build,
    bench_scoped_resolver_selection,
    bench_symbols,
    bench_workspace,
    bench_react_traits,
    bench_aggregate_and_multi_report,
    bench_finite_set_membership,
    bench_impacted_checks,
    bench_observer_overhead,
    bench_relationship_projection,
);
criterion_main!(benches);
