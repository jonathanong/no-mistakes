use super::*;
use crate::benchmark_support::{
    language_frontend_fixture, shard_should_run, FRONTEND_CACHE_LIFETIMES, FRONTEND_CACHE_ORIGINAL,
    GENERAL_MEMORY, LANGUAGE_FRONTENDS,
};

#[test]
fn cache_lifecycle_uses_real_saved_frontend_fixture_and_all_four_workers() {
    let mut lifecycle = FrontendCacheLifecycle::new(language_frontend_fixture());
    assert_eq!(lifecycle.cold_collections, 0);
    assert!(lifecycle.broadcast_workers.is_empty());
    assert_eq!(lifecycle.warm_collections, 0);
    let expected = LanguageFrontendSummary {
        files: 117,
        parsed_files: 69,
        imports: 66,
        enqueues: 7,
        workers: 6,
        route_handlers: 41,
        ..LanguageFrontendSummary::default()
    };
    // Unit tests prove the work boundary/counts, not fresh-process allocation.
    assert_eq!(lifecycle.cold(), expected);
    assert_eq!(lifecycle.cold_collections, 1);
    assert_eq!(lifecycle.exercise_workers(), vec![expected; 4]);
    assert_eq!(lifecycle.broadcast_workers, vec![0, 1, 2, 3]);
    assert_eq!(lifecycle.warm(), expected);
    assert_eq!(lifecycle.warm_collections, 1);
}

#[test]
fn cache_lifetimes_is_explicit_only_and_excludes_original_preflights() {
    for selected in [
        None,
        Some(""),
        Some(GENERAL_MEMORY),
        Some(LANGUAGE_FRONTENDS),
    ] {
        assert_eq!(
            shard_should_run(FRONTEND_CACHE_LIFETIMES, selected),
            Ok(false)
        );
    }
    assert_eq!(
        shard_should_run(FRONTEND_CACHE_LIFETIMES, Some(FRONTEND_CACHE_LIFETIMES)),
        Ok(true)
    );
    assert_eq!(
        shard_should_run(LANGUAGE_FRONTENDS, Some(FRONTEND_CACHE_LIFETIMES)),
        Ok(false)
    );
}

#[test]
fn original_alias_is_explicit_only_and_isolates_the_same_workload_registration() {
    for selected in [
        None,
        Some(""),
        Some(GENERAL_MEMORY),
        Some(LANGUAGE_FRONTENDS),
        Some(FRONTEND_CACHE_LIFETIMES),
    ] {
        assert_eq!(
            shard_should_run(FRONTEND_CACHE_ORIGINAL, selected),
            Ok(false)
        );
    }
    assert_eq!(
        shard_should_run(FRONTEND_CACHE_ORIGINAL, Some(FRONTEND_CACHE_ORIGINAL)),
        Ok(true)
    );
    assert_eq!(
        shard_should_run(LANGUAGE_FRONTENDS, Some(FRONTEND_CACHE_ORIGINAL)),
        Ok(false)
    );
    assert_eq!(
        shard_should_run(FRONTEND_CACHE_LIFETIMES, Some(FRONTEND_CACHE_ORIGINAL)),
        Ok(false)
    );
}
