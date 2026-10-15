//! CPU CodSpeed shards by product surface. Unset/`general-memory` run every
//! non-production workload except explicit frontend cache controls. Unknown
//! values fail fast so a CI typo cannot run
//! the full suite.

pub(super) use no_mistakes::benchmark_support::{
    CHECK, FRONTEND_CACHE_LIFETIMES, FRONTEND_CACHE_ORIGINAL, GENERAL_MEMORY, GRAPH_CORE,
    GRAPH_FINALIZATION, GRAPH_GATES, GRAPH_PRODUCTION, LANGUAGE_FRONTENDS, NATIVE_FRONTENDS,
    OBSERVER, QUERY, TESTS_PLAN,
};

pub(super) fn should_run(shard: &str) -> bool {
    match no_mistakes::benchmark_support::shard_should_run(
        shard,
        std::env::var("NO_MISTAKES_BENCH_SHARD").ok().as_deref(),
    ) {
        Ok(run) => run,
        Err(err) => panic!("{err}"),
    }
}
