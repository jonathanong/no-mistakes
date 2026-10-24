use super::super::SourceStore;
use crate::codebase::ts_source::FileInventory;
use crate::config::source_locations::ConfigPath;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config/source-locations")
        .join(name)
}

#[test]
fn locations_are_lazy_and_share_existing_source_reads() {
    let path = fixture("block.yaml");
    let store = SourceStore::new(Arc::new(FileInventory::from_paths(std::slice::from_ref(
        &path,
    ))));
    assert!(store.config_locations.get().is_none());
    store.read_path(&path).unwrap();
    assert!(store.config_locations.get().is_none());
    let first = store.config_locations(&path).unwrap();
    let second = store
        .config_locations(&path.parent().unwrap().join("./block.yaml"))
        .unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(store.physical_read_count(), 1);
    assert_eq!(store.config_locations.get().unwrap().len(), 1);
    assert_eq!(first.line(&ConfigPath::default().key("projects")), Some(2));
}

#[test]
fn concurrent_supplemental_location_requests_compute_once() {
    let path = fixture("comments.jsonc");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let store = SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(&[])),
        Some(Arc::clone(&observer)),
    );
    let barrier = Barrier::new(8);
    let locations = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    store.config_locations(&path).unwrap()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(locations
        .iter()
        .all(|current| Arc::ptr_eq(current, &locations[0])));
    assert_eq!(store.physical_read_count(), 1);
    assert_eq!(store.inventory().len(), 0);
    assert_eq!(observer.snapshot().work["config_locations.parses"], 1);
}

#[test]
fn missing_malformed_and_unsupported_location_results_are_cached() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let store = SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(&[])),
        Some(Arc::clone(&observer)),
    );
    for name in [
        "missing.yaml",
        "invalid.yaml",
        "invalid.jsonc",
        "unknown.toml",
    ] {
        let path = fixture(name);
        assert!(store.config_locations(&path).is_none());
        assert!(store.config_locations(&path).is_none());
        // A cached None must be retained just like a successful parse.
        let path = crate::codebase::ts_source::normalize_discovery_path(&path);
        let slots = store.config_locations.get().unwrap();
        assert!(matches!(slots.get(&path).unwrap().get(), Some(None)));
    }
    assert_eq!(store.physical_read_count(), 4);
    assert_eq!(observer.snapshot().work["config_locations.parses"], 4);
}
