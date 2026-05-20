use crate::pipeline::cache::Cache;
use crate::report::types::FetchSide;
use no_mistakes_core::fetch::file_analysis::analyze_file;
use std::collections::{HashMap, HashSet};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_analyze_file_use_server_overrides_inherited_client_state() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("helper.ts");
    fs::write(&file, "'use server';\nfetch('/api/server');").unwrap();

    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    let mut visited = HashSet::new();
    let mut fetches = Vec::new();
    let is_client = analyze_file(
        &file,
        dir.path(),
        &mut visited,
        &mut fetches,
        &mut cache,
        true,
        false,
    )
    .unwrap();
    assert!(!is_client);
    assert_eq!(fetches.len(), 1);
    assert_eq!(fetches[0].side, FetchSide::Server);
}

#[test]
fn test_analyze_file_imported_file_is_analyzed() {
    let dir = tempdir().unwrap();
    let helper = dir.path().join("helper.ts");
    fs::write(&helper, "export const helper = () => fetch('/api/helper');").unwrap();
    let file = dir.path().join("file.ts");
    fs::write(&file, "import { helper } from './helper';\nhelper();").unwrap();

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
    assert_eq!(fetches[0].path, "/api/helper");
}

#[test]
fn test_analyze_file_ignores_unused_imported_helper() {
    let dir = tempdir().unwrap();
    let helper = dir.path().join("helper.ts");
    fs::write(&helper, "export const helper = () => fetch('/api/helper');").unwrap();
    let file = dir.path().join("file.ts");
    fs::write(
        &file,
        "import { helper } from './helper';\nfetch('/api/file');",
    )
    .unwrap();

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
    assert_eq!(fetches[0].path, "/api/file");
}

#[test]
fn test_analyze_file_side_effect_import_is_analyzed() {
    let dir = tempdir().unwrap();
    let helper = dir.path().join("helper.ts");
    fs::write(&helper, "fetch('/api/helper');").unwrap();
    let file = dir.path().join("file.ts");
    fs::write(&file, "import './helper';\nfetch('/api/file');").unwrap();

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

    assert_eq!(fetches.len(), 2);
    assert_eq!(fetches[0].path, "/api/file");
    assert_eq!(fetches[1].path, "/api/helper");
}
