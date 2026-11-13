use super::*;

fn key(file: &str, scope: usize) -> ParameterKey {
    ParameterKey {
        file: file.into(),
        scope,
        name: "query".into(),
        invocation_read: None,
        initializing: Vec::new(),
    }
}

#[test]
fn published_parameter_stops_wait_chain_before_waiter_resumes() {
    let cache = ParameterCache::default();
    let caller = key("caller.mts", 1);
    let stage = key("stages-00.mts", 3);
    let publisher = std::thread::current().id();
    let CacheAccess::Compute(caller_lease) = cache.acquire(caller.clone()) else {
        panic!("caller parameter must be cold");
    };
    let (owned, owner) = mpsc::channel();
    let (release, released) = mpsc::channel();
    std::thread::scope(|scope| {
        let worker_cache = &cache;
        let worker = scope.spawn(move || {
            let CacheAccess::Compute(stage_lease) = worker_cache.acquire(stage) else {
                panic!("stage parameter must be cold");
            };
            owned.send(std::thread::current().id()).unwrap();
            released.recv().unwrap();
            drop(stage_lease);
        });
        let waiter = owner.recv().unwrap();
        {
            let mut state = cache.state.lock().unwrap();
            // Model the waiter paused after registering its dependency, before resume.
            state.waiting.insert(waiter, caller.clone());
            assert!(waits_for(&state, waiter, publisher));
        }
        caller_lease.publish(
            Arc::new(CachedParameter {
                value: FlowValue::Unknown,
                origins: None,
            }),
            (caller.file.clone(), CallableId(1)),
            fx_set(),
        );
        let cycle = {
            let state = cache.state.lock().unwrap();
            assert!(matches!(state.entries.get(&caller), Some(Entry::Ready(_))));
            assert!(state.waiting.contains_key(&waiter));
            waits_for(&state, waiter, publisher)
        };
        release.send(()).unwrap();
        worker.join().unwrap();
        assert!(
            !cycle,
            "a published caller parameter cannot block its waiter"
        );
    });
}

#[test]
fn same_thread_recursive_parameter_still_returns_cycle() {
    let cache = ParameterCache::default();
    let parameter = key("caller.mts", 1);
    let CacheAccess::Compute(lease) = cache.acquire(parameter.clone()) else {
        panic!("parameter must be cold");
    };
    assert!(matches!(cache.acquire(parameter), CacheAccess::Cycle));
    drop(lease);
}
