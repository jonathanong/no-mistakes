use super::*;

#[test]
fn normalizes_file_universe_order_and_duplicates() {
    let a = PathBuf::from("/repo/a.ts");
    let b = PathBuf::from("/repo/b.ts");
    let first = graph::GraphFiles::from_files(vec![b.clone(), a.clone(), a.clone()]);
    let second = graph::GraphFiles::from_files(vec![a, b]);

    assert_eq!(
        GraphFileUniverseKey::new(&first, 0),
        GraphFileUniverseKey::new(&second, 0)
    );
    assert_ne!(
        GraphFileUniverseKey::new(&first, 0),
        GraphFileUniverseKey::new(&first, 1)
    );
}

#[test]
fn memoizes_successful_and_failed_builds() {
    let cache = SharedBuildCache::<u8, u8>::default();
    assert_eq!(*cache.get_or_build(1, || Ok(7)).unwrap(), 7);
    assert_eq!(*cache.get_or_build(1, || Ok(9)).unwrap(), 7);

    let first = cache
        .get_or_build(2, || anyhow::bail!("first failure"))
        .unwrap_err();
    let second = cache.get_or_build(2, || Ok(9)).unwrap_err();
    assert_eq!(first.to_string(), "first failure");
    assert_eq!(second.to_string(), "first failure");
    assert_eq!(cache.build_count(), 2);
}

#[test]
fn concurrent_callers_run_one_build() {
    let cache = std::sync::Arc::new(SharedBuildCache::<u8, u8>::default());
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let threads = (0..8)
        .map(|_| {
            let cache = cache.clone();
            let calls = calls.clone();
            std::thread::spawn(move || {
                cache
                    .get_or_build(1, || {
                        calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        Ok(7)
                    })
                    .unwrap()
            })
        })
        .collect::<Vec<_>>();

    for thread in threads {
        assert_eq!(*thread.join().unwrap(), 7);
    }
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
    assert_eq!(cache.build_count(), 1);
}
