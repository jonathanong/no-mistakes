use super::*;
use crate::codebase::postgres::query_annotation::module_ids::tests::{saved_paths, HashBytes};
use std::hash::Hash;

#[test]
fn actual_snapshot_cache_keys_hash_no_path_bytes_and_retain_shared_identities() {
    let paths = saved_paths();
    let ids = Arc::new(ModuleIds::new(paths.clone()));
    let mut cache = ModuleCache::with_ids(Arc::clone(&ids));
    for (frame, path) in paths.iter().enumerate() {
        cache.insert(ids.id(path), frame);
    }
    let snapshot = cache.clone();
    assert!(Arc::ptr_eq(
        cache.ids.get().unwrap(),
        snapshot.ids.get().unwrap()
    ));
    let mut work = HashBytes::default();
    for key in cache.keys() {
        key.hash(&mut work);
    }
    assert_eq!(work.0, paths.len() * std::mem::size_of::<usize>());
    // Frame compaction changes frame values, never the immutable module key.
    for frame in cache.values_mut() {
        *frame += 7;
    }
    for (frame, path) in paths.iter().enumerate() {
        assert_eq!(cache[&ids.id(path)], frame + 7);
    }
    cache.clone_from(&snapshot);
    for (frame, path) in paths.iter().enumerate() {
        assert_eq!(cache[&ids.id(path)], frame);
    }
    assert!(Arc::ptr_eq(cache.ids.get().unwrap(), &ids));
}
