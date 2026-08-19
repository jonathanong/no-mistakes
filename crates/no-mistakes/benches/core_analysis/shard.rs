//! CPU CodSpeed shards by product surface. Unset/`general-memory` run everything
//! (except production-graph, which `graph.rs` still skips on `general-memory`).
//!
//! - `check`: aggregate `check`, observer overhead, React traits
//! - `tests-plan`: impacted-checks (test-plan generation)
//! - `graph`: facts, graph build/query/finalization, language-frontend extract
//! - `query`: symbols, workspace, indexes, relationship projection, multi-report

pub(super) const CHECK: &str = "check";
pub(super) const TESTS_PLAN: &str = "tests-plan";
pub(super) const GRAPH: &str = "graph";
pub(super) const QUERY: &str = "query";

const CPU_SHARDS: &[&str] = &[CHECK, TESTS_PLAN, GRAPH, QUERY];

pub(super) fn should_run(shard: &str) -> bool {
    match std::env::var("NO_MISTAKES_BENCH_SHARD") {
        Ok(current) if CPU_SHARDS.contains(&current.as_str()) => current == shard,
        _ => true,
    }
}
