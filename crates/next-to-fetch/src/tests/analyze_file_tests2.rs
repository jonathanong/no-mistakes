use crate::analyze::file::analyze_file;
use crate::pipeline::cache::Cache;
use std::collections::{HashMap, HashSet};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_analyze_file_not_exists() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("missing.ts");
    let mut visited = HashSet::new();
    let mut fetches = Vec::new();
    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    analyze_file(
        &missing,
        dir.path(),
        &mut visited,
        &mut fetches,
        &mut cache,
        false,
        false,
    )
    .unwrap();
    assert!(fetches.is_empty());
}

#[test]
fn test_analyze_file_already_visited() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("file.ts");
    fs::write(&file, "").unwrap();

    let mut visited = HashSet::new();
    visited.insert((file.canonicalize().unwrap(), false, false));
    let mut fetches = Vec::new();
    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
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
    assert!(fetches.is_empty());
}

#[test]
fn test_analyze_file_read_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("dir.ts");
    fs::create_dir(&path).unwrap();
    let mut visited = HashSet::new();
    let mut fetches = Vec::new();
    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    let err = analyze_file(
        &path,
        dir.path(),
        &mut visited,
        &mut fetches,
        &mut cache,
        false,
        false,
    )
    .err()
    .unwrap();
    assert!(
        err.to_string().contains("failed to read") || err.to_string().contains("Is a directory")
    );
}

#[test]
fn test_route_reaches_target_client_file() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("client.ts");
    fs::write(
        &file,
        "
            'use client';
            import { helper } from './helper';
            helper();
            export {};
            ",
    )
    .unwrap();

    let helper = dir.path().join("helper.ts");
    fs::write(&helper, "export const helper = () => fetch('/api/helper');").unwrap();

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
    assert!(fetches.iter().any(|fetch| fetch.path == "/api/helper"));
    assert!(fetches.iter().any(|fetch| !fetch.rsc));
}

#[test]
fn test_analyze_file_imported_file_parse_error_is_propagated() {
    let root = tempdir().unwrap();
    fs::write(
        root.path().join("file.ts"),
        "import { helper } from './helper';\nhelper();",
    )
    .unwrap();
    fs::write(root.path().join("helper.ts"), "export const = invalid;").unwrap();

    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    let mut visited = HashSet::new();
    let mut fetches = Vec::new();

    let err = analyze_file(
        &root.path().join("file.ts"),
        root.path(),
        &mut visited,
        &mut fetches,
        &mut cache,
        false,
        false,
    )
    .unwrap_err();
    assert!(err.to_string().contains("parse") || err.to_string().contains("expected"));
}
