//! Parse `NO_MISTAKES_BENCH_SHARD` for the core_analysis harness.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchShard {
    All,
    Named(&'static str),
}

pub const CHECK: &str = "check";
pub const OBSERVER: &str = "observer";
pub const TESTS_PLAN: &str = "tests-plan";
pub const GRAPH_CORE: &str = "graph-core";
pub const GRAPH_GATES: &str = "graph-gates";
pub const LANGUAGE_FRONTENDS: &str = "language-frontends";
pub const NATIVE_FRONTENDS: &str = "native-frontends";
pub const GRAPH_FINALIZATION: &str = "graph-finalization";
pub const GRAPH_PRODUCTION: &str = "graph-production";
pub const QUERY: &str = "query";
pub const GENERAL_MEMORY: &str = "general-memory";

const NAMED_SHARDS: &[&str] = &[
    CHECK,
    OBSERVER,
    TESTS_PLAN,
    GRAPH_CORE,
    GRAPH_GATES,
    LANGUAGE_FRONTENDS,
    NATIVE_FRONTENDS,
    GRAPH_FINALIZATION,
    GRAPH_PRODUCTION,
    QUERY,
];

pub fn parse_bench_shard(value: Option<&str>) -> Result<BenchShard, String> {
    match value {
        None | Some("") | Some(GENERAL_MEMORY) => Ok(BenchShard::All),
        Some(name) => NAMED_SHARDS
            .iter()
            .copied()
            .find(|known| *known == name)
            .map(BenchShard::Named)
            .ok_or_else(|| {
                format!(
                    "unknown NO_MISTAKES_BENCH_SHARD={name:?}; expected unset, {GENERAL_MEMORY}, or one of {NAMED_SHARDS:?}"
                )
            }),
    }
}

pub fn shard_should_run(requested: &str, current: Option<&str>) -> Result<bool, String> {
    Ok(match parse_bench_shard(current)? {
        BenchShard::All => true,
        BenchShard::Named(name) => name == requested,
    })
}
