use super::{dispatch, install, worker_limit};

#[test]
fn source_worker_limit_prefers_jobs_then_env_then_recorded_pool() {
    assert_eq!(worker_limit(4, Some("8"), None), 4);
    assert_eq!(worker_limit(0, Some("8"), None), 8);
    for value in [None, Some("0"), Some("invalid"), Some("-1")] {
        assert_eq!(worker_limit(0, value, None), num_cpus::get());
    }
    assert_eq!(worker_limit(8, None, Some(2)), 2);
    assert_eq!(worker_limit(1, Some("8"), Some(4)), 1);
    assert_eq!(worker_limit(0, Some("8"), Some(3)), 3);
}

#[test]
fn sql_batch_pool_respects_jobs_and_does_not_exceed_recorded_pool() {
    // Matching recorded size runs on the caller without building another pool.
    // A fake recorded value must not touch Rayon, or this test pins the global pool.
    assert_eq!(dispatch(4, Some(4), || 4), 4);
    assert_eq!(dispatch(2, None, rayon::current_num_threads), 2);
    assert_eq!(dispatch(2, Some(4), rayon::current_num_threads), 2);
    // `install` applies the same limit. A closure that ignores Rayon stays
    // order-independent when another test has already recorded the global pool.
    assert_eq!(install(4, || 4), 4);
    let capped = worker_limit(8, None, Some(2));
    assert_eq!(capped, 2);
    assert!(capped <= 8);
}
