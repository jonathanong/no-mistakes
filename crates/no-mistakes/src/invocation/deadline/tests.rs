use super::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct ReleasePhase<'a>(&'a AtomicUsize);

impl Drop for ReleasePhase<'_> {
    fn drop(&mut self) {
        // Unblock workers if the test fails before the restore phase.
        self.0.store(3, Ordering::Release);
    }
}

#[test]
fn committed_output_disables_later_deadline_checks() {
    let _serial = super::super::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let guard = DeadlineGuard::install_for_invocation(
        Some(Duration::from_secs(30)),
        Some(std::thread::current().id()),
    )
    .unwrap();

    commit_timeout().unwrap();
    commit_timeout().unwrap();
    active_deadline()
        .write()
        .unwrap()
        .as_mut()
        .unwrap()
        .expires_at = Instant::now();

    check_timeout().unwrap();
    assert_eq!(remaining_timeout().unwrap(), None);
    drop(guard);
}

#[test]
fn expired_deadline_cannot_be_committed() {
    let _serial = super::super::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let previous = active_deadline().write().unwrap().replace(Deadline {
        expires_at: Instant::now(),
        timeout: Duration::from_secs(1),
        owner: Some(std::thread::current().id()),
        committed: false,
    });

    assert!(commit_timeout().is_err());
    assert_eq!(
        super::timeout_diagnostic(Duration::from_secs(1), "check.parse"),
        "command timed out after 1 seconds during check.parse"
    );

    *active_deadline().write().unwrap() = previous;
}

#[test]
fn disabled_cli_timeout_does_not_start_a_watchdog() {
    let _serial = super::super::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let guard = DeadlineGuard::install_for_cli(None).unwrap();
    assert!(guard.cancel_watch.is_none());
}

#[test]
fn replaced_deadline_lets_the_watchdog_return() {
    let _serial = super::super::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let expires_at = Instant::now()
        .checked_sub(Duration::from_secs(1))
        .expect("past instant");
    // The expired instant is not the active deadline, so the watcher must return
    // instead of exiting the process.
    super::watch::watch_deadline(
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        expires_at,
        Duration::from_secs(1),
        None,
    );
}

#[test]
fn invocation_deadline_limits_command_without_extending_a_shorter_phase_timeout() {
    let _serial = super::super::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let limit = super::Deadline {
        expires_at: super::clock::now() + Duration::from_secs(5),
        timeout: Duration::from_secs(5),
        owner: None,
        committed: false,
    };
    let guard = super::DeadlineGuard::install_for_invocation_with_deadline(
        Some(Duration::from_secs(20)),
        None,
        Some(limit),
    )
    .unwrap();
    assert_eq!(
        super::active_deadline().read().unwrap().unwrap().expires_at,
        limit.expires_at
    );
    drop(guard);
    let guard = super::DeadlineGuard::install_for_invocation_with_deadline(
        Some(Duration::from_secs(1)),
        None,
        Some(limit),
    )
    .unwrap();
    assert!(super::active_deadline().read().unwrap().unwrap().expires_at < limit.expires_at);
    drop(guard);
    assert_eq!(
        super::timeout_diagnostic(Duration::from_millis(3500), "invocation.lock"),
        "command timed out after 3500 milliseconds during invocation.lock"
    );
}

#[test]
fn expired_command_keeps_its_phase_when_the_invocation_limit_is_live() {
    let _serial = super::super::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let limit = Deadline {
        expires_at: clock::now() + Duration::from_secs(30),
        timeout: Duration::from_secs(30),
        owner: Some(std::thread::current().id()),
        committed: false,
    };
    let previous_phase = current_phase();
    set_timeout_phase("command");
    let outcome = DeadlineGuard::install_for_invocation_with_deadline(
        Some(Duration::ZERO),
        Some(std::thread::current().id()),
        Some(limit),
    )
    .map(|guard| {
        // Other tests stream subprocess output concurrently; this deliberately
        // expired test deadline must only affect its owning test thread.
        let foreign_timeout = std::thread::spawn(remaining_timeout).join().unwrap();
        assert_eq!(foreign_timeout.unwrap(), None);
        let result = check_timeout();
        drop(guard);
        result
    });
    set_timeout_phase(previous_phase);
    let error = outcome
        .expect("live invocation limit must allow the command guard")
        .expect_err("zero-duration native command must already be expired");
    assert_eq!(
        error.to_string(),
        "command timed out after 0 seconds during command"
    );
    let expired = Deadline {
        expires_at: clock::now(),
        ..limit
    };
    let error = DeadlineGuard::install_for_invocation_with_deadline(
        Some(Duration::from_secs(1)),
        None,
        Some(expired),
    )
    .err()
    .expect("already-expired invocation must fail before publication");
    assert_eq!(
        error.to_string(),
        "invocation deadline elapsed during lock acquisition"
    );
}

#[test]
fn parallel_workers_observe_expiry_and_not_a_stale_deadline() {
    let _serial = super::super::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // `clock::now` is `Instant::now` and is not injectable. `owner: None` applies
    // the deadline to every worker; a thread owner would hide a stale snapshot.
    const WORKERS: usize = 8;
    let phase = Arc::new(AtomicUsize::new(0));
    let started = Arc::new(AtomicUsize::new(0));
    let saw_expiry = Arc::new(AtomicUsize::new(0));
    let saw_stale = Arc::new(AtomicUsize::new(0));
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(WORKERS)
        .build()
        .expect("worker pool");
    let phase_workers = Arc::clone(&phase);
    let started_workers = Arc::clone(&started);
    let saw_expiry_workers = Arc::clone(&saw_expiry);
    let saw_stale_workers = Arc::clone(&saw_stale);

    std::thread::scope(|scope| {
        scope.spawn(|| {
            pool.install(|| {
                (0..WORKERS).into_par_iter().for_each(|_| {
                    started_workers.fetch_add(1, Ordering::Release);
                    while phase_workers.load(Ordering::Acquire) == 0 {
                        // Hammer the snapshot while the deadline is installed.
                        let _ignored = check_timeout().is_err();
                        std::thread::yield_now();
                    }
                    let began = Instant::now();
                    loop {
                        if phase_workers.load(Ordering::Acquire) >= 3 {
                            break;
                        }
                        if check_timeout().is_err() {
                            saw_expiry_workers.fetch_add(1, Ordering::Release);
                            break;
                        }
                        if began.elapsed() > Duration::from_secs(5) {
                            break;
                        }
                        std::hint::spin_loop();
                    }
                    while phase_workers.load(Ordering::Acquire) < 3 {
                        std::thread::yield_now();
                    }
                    for _ in 0..128 {
                        if check_timeout().is_err() {
                            saw_stale_workers.fetch_add(1, Ordering::Release);
                            break;
                        }
                    }
                });
            });
        });

        let _release = ReleasePhase(&phase);
        let wait_started = Instant::now();
        while started.load(Ordering::Acquire) < WORKERS {
            assert!(
                wait_started.elapsed() < Duration::from_secs(5),
                "workers did not start polling"
            );
            std::thread::yield_now();
        }
        let guard =
            DeadlineGuard::install_for_invocation(Some(Duration::from_millis(50)), None).unwrap();
        phase.store(1, Ordering::Release);
        let wait_expiry = Instant::now();
        while saw_expiry.load(Ordering::Acquire) < WORKERS {
            assert!(
                wait_expiry.elapsed() < Duration::from_secs(5),
                "workers that observed expiry: {}",
                saw_expiry.load(Ordering::Acquire)
            );
            std::thread::yield_now();
        }
        drop(guard);
    });
    assert_eq!(
        saw_stale.load(Ordering::Acquire),
        0,
        "restored deadline must not stay visible"
    );
}
