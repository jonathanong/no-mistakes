use crate::fetches::analyze::resolve::{relative_string, resolve_import};
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-fetches")
        .join(name)
        .join("fixture")
}

#[test]
fn test_resolve_import_index() {
    let root = fixture("resolve-import-index");
    let current = root.join("main.ts");
    let resolved = resolve_import(&current, "./lib").unwrap();
    assert!(resolved.ends_with("lib/index.ts"));
}

#[test]
fn test_resolve_import_explicit_extension() {
    let root = fixture("resolve-import-explicit-extension");
    let file = root.join("lib.ts");
    let current = root.join("main.ts");
    let resolved = resolve_import(&current, "./lib.ts").unwrap();
    assert_eq!(
        resolved.canonicalize().unwrap(),
        file.canonicalize().unwrap()
    );
}

#[test]
fn test_resolve_import_skips_non_javascript_file() {
    let current = fixture("resolve-import-skips-stylesheet").join("main.ts");
    assert_eq!(resolve_import(&current, "./styles"), None);
}

#[test]
fn test_resolve_import_skips_existing_non_javascript_file_without_extension() {
    let current = fixture("resolve-import-skips-extensionless").join("main.ts");
    assert_eq!(resolve_import(&current, "./legacy"), None);
}

#[test]
fn test_resolve_import_root() {
    assert_eq!(resolve_import(Path::new("page.ts"), "./lib"), None);
}

#[test]
fn test_resolve_import_non_dot_specifier() {
    let dir = tempdir().unwrap();
    let current = dir.path().join("main.ts");

    assert_eq!(resolve_import(&current, "react"), None);
}

#[test]
fn test_resolve_import_none() {
    let dir = tempdir().unwrap();
    let current = dir.path().join("main.ts");
    assert_eq!(resolve_import(&current, "./missing"), None);
}

#[test]
fn test_relative_string_failure() {
    let root = Path::new("/root/a");
    let path = Path::new("/root/b");
    assert_eq!(relative_string(root, path), "/root/b");
}
