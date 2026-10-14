use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Barrier, mpsc};
use std::time::Duration;

#[test]
fn independent_type_origins_resolve_without_holding_map_shards() {
    let cache = ParameterCache::default();
    // Three blocked resolutions in two shards also protect colliding keys.
    cache
        .type_origins
        .set(DashMap::with_hasher_and_shard_amount(FxBuildHasher, 2))
        .unwrap();
    let release = (Mutex::new(false), Condvar::new());
    let (entered, arrivals) = mpsc::channel();
    std::thread::scope(|scope| {
        for index in 0..3 {
            let cache = &cache;
            let release = &release;
            let entered = entered.clone();
            scope.spawn(move || {
                let result = cache.type_origin((format!("{index}.ts").into(), "Type".into()), || {
                    entered.send(()).unwrap();
                    let (lock, ready) = release;
                    let mut released = lock.lock().unwrap();
                    while !*released {
                        released = ready.wait(released).unwrap();
                    }
                    ("package".into(), format!("Type{index}"))
                });
                assert_eq!(result, ("package".into(), format!("Type{index}")));
            });
        }
        let arrived = (0..3)
            .filter(|_| arrivals.recv_timeout(Duration::from_secs(5)).is_ok())
            .count();
        *release.0.lock().unwrap() = true;
        release.1.notify_all();
        assert_eq!(arrived, 3, "cold independent origins serialized");
    });
}

#[test]
fn concurrent_type_origin_readers_compute_once_and_survive_generation_clear() {
    let cache = ParameterCache::default();
    assert!(cache.type_origins.get().is_none());
    let computations = AtomicUsize::new(0);
    let start = Barrier::new(8);
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                start.wait();
                assert_eq!(
                    cache.type_origin(("index.ts".into(), "Unknown".into()), || {
                        computations.fetch_add(1, Ordering::Relaxed);
                        ("unresolved".into(), "Unknown".into())
                    }),
                    ("unresolved".into(), "Unknown".into())
                );
            });
        }
    });
    cache.clear();
    assert_eq!(
        cache.type_origin(("index.ts".into(), "Unknown".into()), || panic!("cached")),
        ("unresolved".into(), "Unknown".into())
    );
    assert_eq!(computations.load(Ordering::Relaxed), 1);
}
