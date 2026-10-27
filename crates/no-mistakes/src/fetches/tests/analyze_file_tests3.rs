use crate::fetches::pipeline::cache::Cache;
use crate::fetches::report::types::FetchSide;
use no_mistakes::fetch::file_analysis::analyze_file;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-fetches")
        .join(name)
        .join("fixture")
}

#[test]
fn test_analyze_file_use_server_overrides_inherited_client_state() {
    let root = fixture("analyze-file-use-server-overrides-client");
    let file = root.join("helper.ts");

    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    let mut visited = HashSet::new();
    let mut fetches = Vec::new();
    let is_client = analyze_file(
        &file,
        &root,
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
    let root = fixture("analyze-file-imported-helper");
    let file = root.join("file.ts");

    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    let mut visited = HashSet::new();
    let mut fetches = Vec::new();
    analyze_file(
        &file,
        &root,
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
    let root = fixture("analyze-file-unused-imported-helper");
    let file = root.join("file.ts");

    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    let mut visited = HashSet::new();
    let mut fetches = Vec::new();
    analyze_file(
        &file,
        &root,
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
    let root = fixture("analyze-file-side-effect-import");
    let file = root.join("file.ts");

    let mut cache = Cache {
        files: HashMap::new(),
        imports: HashMap::new(),
    };
    let mut visited = HashSet::new();
    let mut fetches = Vec::new();
    analyze_file(
        &file,
        &root,
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
