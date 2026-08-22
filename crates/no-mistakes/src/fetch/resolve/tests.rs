use super::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_temp_file(content: &str) -> NamedTempFile {
    let mut file = tempfile::Builder::new().suffix(".tsx").tempfile().unwrap();
    write!(file, "{}", content).unwrap();
    file
}

#[test]
fn test_is_client_route_file_non_existent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("missing.tsx");
    assert!(!is_client_route_file(&path).unwrap());
}

#[test]
fn test_is_client_route_file_with_use_client_double_quotes() {
    let file = create_temp_file("\"use client\";\n\nexport default function Page() {}");

    assert!(is_client_route_file(file.path()).unwrap());
}

#[test]
fn test_is_client_route_file_without_use_client() {
    let file = create_temp_file("export default function Page() {}");

    assert!(!is_client_route_file(file.path()).unwrap());
}

#[test]
fn test_is_client_route_file_with_use_client_single_quotes() {
    let file = create_temp_file("'use client';\n\nexport default function Page() {}");

    assert!(is_client_route_file(file.path()).unwrap());
}

#[test]
fn test_is_client_route_file_invalid_syntax() {
    let file = create_temp_file("const const const;");
    assert!(is_client_route_file(file.path()).is_err());
}

#[test]
fn is_client_route_file_with_sources_reuses_the_prepared_store() {
    use crate::codebase::ts_source::{FileInventory, SourceStore};
    use std::sync::Arc;

    let file = create_temp_file("\"use client\";\n\nexport default function Page() {}");
    let path = file.path().to_path_buf();
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&path)));
    let store = SourceStore::new(inventory);

    assert!(is_client_route_file_with_sources(&path, Some(&store)).unwrap());
    let reads = store.physical_read_count();
    assert!(reads >= 1);
    assert!(is_client_route_file_with_sources(&path, Some(&store)).unwrap());
    assert_eq!(store.physical_read_count(), reads);
}

#[test]
fn fetch_resolve_source_does_not_read_the_filesystem_directly() {
    let source = include_str!("../resolve.rs");
    assert!(!source.contains("std::fs::read_to_string"));
    assert!(source.contains("read_prepared_or_open"));
}
