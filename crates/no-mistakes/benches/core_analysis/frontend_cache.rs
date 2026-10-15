#[cfg(codspeed)]
use super::language_frontends::*;
use super::shard;
use criterion::Criterion;
#[cfg(codspeed)]
use criterion::{black_box, IterManualOptions};
#[cfg(codspeed)]
use no_mistakes::benchmark_support::{
    language_frontend_fixture, FrontendCacheLifecycle, LanguageFrontendSummary,
};

#[cfg(codspeed)]
fn verify(summary: LanguageFrontendSummary) {
    assert_eq!(
        summary,
        LanguageFrontendSummary {
            files: EXPECTED_LANGUAGE_FRONTEND_FILES,
            parsed_files: EXPECTED_LANGUAGE_FRONTEND_PARSED,
            imports: EXPECTED_LANGUAGE_FRONTEND_IMPORTS,
            enqueues: EXPECTED_LANGUAGE_FRONTEND_ENQUEUES,
            workers: EXPECTED_LANGUAGE_FRONTEND_WORKERS,
            route_handlers: EXPECTED_LANGUAGE_FRONTEND_ROUTES,
            ..LanguageFrontendSummary::default()
        }
    );
}

#[cfg(codspeed)]
pub(super) fn bench_frontend_cache_lifetimes(c: &mut Criterion) {
    if !shard::should_run(shard::FRONTEND_CACHE_LIFETIMES) {
        return;
    }
    // This group is registered FIRST and only in its explicit shard. No
    // frontend collection/preflight may precede its cold measurement.
    let mut lifecycle = FrontendCacheLifecycle::new(language_frontend_fixture());
    let mut group = c.benchmark_group("frontend_cache_lifetimes");
    group.bench_function("cold_frontend_collection", |b| {
        let mut observed = None;
        b.iter_manual_unstable(IterManualOptions::new(), || {
            observed = Some(black_box(lifecycle.cold()));
        });
        verify(observed.expect("cold measurement did not execute"));
        println!(
            "frontend-cache-ledger cold={} broadcast={:?} warm={}",
            lifecycle.cold_collections, lifecycle.broadcast_workers, lifecycle.warm_collections
        );
    });
    group.bench_function("warm_after_worker_broadcast", |b| {
        for summary in lifecycle.exercise_workers() {
            verify(summary);
        }
        let mut observed = None;
        b.iter_manual_unstable(IterManualOptions::new(), || {
            observed = Some(black_box(lifecycle.warm()));
        });
        verify(observed.expect("warm measurement did not execute"));
        println!(
            "frontend-cache-ledger cold={} broadcast={:?} warm={}",
            lifecycle.cold_collections, lifecycle.broadcast_workers, lifecycle.warm_collections
        );
    });
    group.finish();
}

#[cfg(not(codspeed))]
pub(super) fn bench_frontend_cache_lifetimes(_c: &mut Criterion) {
    if shard::should_run(shard::FRONTEND_CACHE_LIFETIMES) {
        eprintln!("frontend-cache-lifetimes unavailable: requires isolated CodSpeed instrumentation; no collections executed");
    }
}
