#[path = "core_analysis/aggregate.rs"]
mod aggregate;
#[path = "core_analysis/call_index.rs"]
mod call_index;
#[path = "core_analysis/extract.rs"]
mod extract;
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
#[path = "core_analysis/postgres_bounds.rs"]
mod postgres_bounds;
#[path = "core_analysis/postgres_scopes.rs"]
mod postgres_scopes;
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
#[path = "core_analysis/sql_fetch.rs"]
mod sql_fetch;
#[path = "core_analysis/sql_source_positions.rs"]
mod sql_source_positions;

use aggregate::{
    bench_aggregate_and_multi_report, bench_finite_set_membership, bench_impacted_checks,
};
use call_index::{bench_call_site_membership, bench_callable_file_index_construction};
use criterion::{criterion_group, criterion_main};
use extract::bench_extract_import_facts;
use graph::{
    bench_facts_graph_and_query, bench_high_fanout_finalization,
    bench_import_only_vs_workspace_relationships, bench_lazy_traversal,
};
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
fn bench_sql_source_positions(c: &mut criterion::Criterion) {
    if shard::should_run(shard::QUERY) {
        sql_source_positions::bench_sql_source_positions(c);
    }
}

fn bench_sql_fetch_fast_path(c: &mut criterion::Criterion) {
    if shard::should_run(shard::QUERY) {
        sql_fetch::bench_sql_fetch_fast_path(c);
    }
}

fn bench_postgres_scopes(c: &mut criterion::Criterion) {
    if shard::should_run(shard::QUERY) {
        postgres_scopes::bench_scopes(c);
    }
}

criterion_group!(
    benches,
    bench_sql_source_positions,
    bench_lazy_traversal,
    postgres_bounds::bench_postgres_bounds,
    bench_import_only_vs_workspace_relationships,
    bench_callable_file_index_construction,
    bench_call_site_membership,
    bench_extract_import_facts,
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
    bench_postgres_scopes,
    bench_relationship_projection,
    bench_sql_fetch_fast_path,
);
criterion_main!(benches);
