use crate::analyze::file::analyze_file;
use crate::pipeline::cache::Cache;
use std::collections::{HashMap, HashSet};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_analyze_file_cache_hit() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("file.ts");
    fs::write(&file, "fetch('/api/cache')").unwrap();

    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    let mut visited = HashSet::new();
    let mut fetches = Vec::new();
    analyze_file(
        &file,
        dir.path(),
        &mut visited,
        &mut fetches,
        &mut cache,
        false,
        false,
    )
    .unwrap();
    assert_eq!(fetches.len(), 1);

    let mut visited2 = HashSet::new();
    let mut fetches2 = Vec::new();
    analyze_file(
        &file,
        dir.path(),
        &mut visited2,
        &mut fetches2,
        &mut cache,
        false,
        false,
    )
    .unwrap();
    assert_eq!(fetches2.len(), 1);
    assert_eq!(fetches2[0].path, "/api/cache");
}

#[test]
fn test_analyze_file_cache_hit_reuses_client_state() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("file.ts");
    fs::write(&file, "'use client'; fetch('/api/cache')").unwrap();

    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    let mut visited = HashSet::new();
    let mut fetches = Vec::new();
    let route_is_client = analyze_file(
        &file,
        dir.path(),
        &mut visited,
        &mut fetches,
        &mut cache,
        false,
        false,
    )
    .unwrap();
    assert!(route_is_client);
    assert_eq!(cache.files.len(), 2);

    let mut visited2 = HashSet::new();
    let mut fetches2 = Vec::new();
    let route_is_client = analyze_file(
        &file,
        dir.path(),
        &mut visited2,
        &mut fetches2,
        &mut cache,
        false,
        false,
    )
    .unwrap();
    assert!(route_is_client);
    assert_eq!(cache.files.len(), 2);
    assert_eq!(fetches2.len(), 1);
    assert_eq!(fetches2[0].path, "/api/cache");
}

#[test]
fn test_analyze_file_cache_hit_reuses_client_state_with_client_flag() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("file.ts");
    fs::write(&file, "'use client'; fetch('/api/cache')").unwrap();

    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    let mut visited = HashSet::new();
    let mut fetches = Vec::new();
    analyze_file(
        &file,
        dir.path(),
        &mut visited,
        &mut fetches,
        &mut cache,
        false,
        false,
    )
    .unwrap();
    assert_eq!(cache.files.len(), 2);

    let mut visited2 = HashSet::new();
    let mut fetches2 = Vec::new();
    let route_is_client = analyze_file(
        &file,
        dir.path(),
        &mut visited2,
        &mut fetches2,
        &mut cache,
        true,
        false,
    )
    .unwrap();
    assert!(route_is_client);
    assert_eq!(cache.files.len(), 2);
    assert_eq!(fetches2.len(), 1);
    assert_eq!(fetches2[0].path, "/api/cache");
}
