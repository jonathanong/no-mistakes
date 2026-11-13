use super::*;
use std::sync::mpsc;

pub(super) type Hook = Arc<dyn Fn(Event, &ParameterKey) + Send + Sync>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Event {
    Computing,
    Waiting,
    Resuming,
    Cycle,
}

#[path = "parameter_cache_fixture_schedule_tests.rs"]
mod fixture;

pub(super) fn configure(cache: &ParameterCache, key: &ParameterKey) {
    if cache.scheduling.get().is_none() {
        if let Some(hook) = fixture::configured(key) {
            let _ = cache.scheduling.set(hook);
        }
    }
}

pub(in super::super) fn max_task_len(root: &std::path::Path) -> usize {
    fixture::max_task_len(root)
}

#[derive(Debug, PartialEq, Eq)]
enum Scheduled {
    CallerOwned,
    CallerWaiting,
    CallerResuming,
    StageWaiting,
    StageResult(bool),
}

fn key(file: &str) -> ParameterKey {
    ParameterKey {
        file: file.into(),
        scope: 1,
        name: "query".into(),
        invocation_read: None,
        initializing: Vec::new(),
    }
}

#[test]
fn resumed_waiter_does_not_create_a_false_acquire_cycle() {
    let cache = ParameterCache::default();
    let caller = key("caller.mts");
    let stage = key("stages-00.mts");
    let (events, scheduled) = mpsc::channel();
    let resume = Arc::new((Mutex::new(false), Condvar::new()));
    let (publish, publish_now) = mpsc::channel();
    let (request, request_now) = mpsc::channel();
    std::thread::scope(|scope| {
        let gate = Arc::clone(&resume);
        let observed = events.clone();
        let watched = caller.clone();
        cache
            .scheduling
            .set(Arc::new(move |event, key| {
                if !matches!(event, Event::Waiting | Event::Resuming) {
                    return;
                }
                if key == &watched {
                    observed
                        .send(if event == Event::Waiting {
                            Scheduled::CallerWaiting
                        } else {
                            Scheduled::CallerResuming
                        })
                        .unwrap();
                    if event == Event::Resuming {
                        let mut released = gate.0.lock().unwrap();
                        while !*released {
                            released = gate.1.wait(released).unwrap();
                        }
                    }
                } else if event == Event::Waiting {
                    observed.send(Scheduled::StageWaiting).unwrap();
                }
            }))
            .ok()
            .expect("fresh hook");
        let owner_cache = &cache;
        let owner_events = events.clone();
        let owner_caller = caller.clone();
        let owner_stage = stage.clone();
        let owner = scope.spawn(move || {
            let CacheAccess::Compute(lease) = owner_cache.acquire(owner_caller) else {
                panic!("cold caller");
            };
            owner_events.send(Scheduled::CallerOwned).unwrap();
            publish_now.recv().unwrap();
            drop(lease);
            request_now.recv().unwrap();
            let ready = matches!(owner_cache.acquire(owner_stage), CacheAccess::Ready(_));
            owner_events.send(Scheduled::StageResult(ready)).unwrap();
        });
        assert_eq!(scheduled.recv().unwrap(), Scheduled::CallerOwned);
        let waiter_cache = &cache;
        let waiter = scope.spawn(move || {
            let CacheAccess::Compute(lease) = waiter_cache.acquire(stage) else {
                panic!("cold stage");
            };
            assert!(matches!(
                waiter_cache.acquire(caller),
                CacheAccess::Ready(_)
            ));
            drop(lease);
        });
        assert_eq!(scheduled.recv().unwrap(), Scheduled::CallerWaiting);
        publish.send(()).unwrap();
        assert_eq!(scheduled.recv().unwrap(), Scheduled::CallerResuming);
        request.send(()).unwrap();
        let observation = scheduled.recv().unwrap();
        *resume.0.lock().unwrap() = true;
        resume.1.notify_all();
        owner.join().unwrap();
        waiter.join().unwrap();
        assert_eq!(observation, Scheduled::StageWaiting);
        assert_eq!(scheduled.recv().unwrap(), Scheduled::StageResult(true));
    });
}

#[test]
fn genuine_cross_worker_parameter_cycle_still_returns_cycle() {
    let cache = ParameterCache::default();
    let caller = key("caller.mts");
    let stage = key("stages-00.mts");
    let (waiting, registered) = mpsc::channel();
    cache
        .scheduling
        .set(Arc::new(move |event, _| {
            if event == Event::Waiting {
                waiting.send(()).unwrap();
            }
        }))
        .ok()
        .expect("fresh hook");
    let CacheAccess::Compute(caller_lease) = cache.acquire(caller.clone()) else {
        panic!("cold caller");
    };
    std::thread::scope(|scope| {
        let worker_cache = &cache;
        let worker_stage = stage.clone();
        let worker = scope.spawn(move || {
            let CacheAccess::Compute(stage_lease) = worker_cache.acquire(worker_stage) else {
                panic!("cold stage");
            };
            assert!(matches!(
                worker_cache.acquire(caller),
                CacheAccess::Ready(_)
            ));
            drop(stage_lease);
        });
        registered.recv().unwrap();
        assert!(matches!(cache.acquire(stage), CacheAccess::Cycle));
        drop(caller_lease);
        worker.join().unwrap();
    });
}
