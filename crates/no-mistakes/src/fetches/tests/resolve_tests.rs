use crate::fetches::analyze::resolve::{relative_string, resolve_import};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_resolve_import_index() {
    let dir = tempdir().unwrap();
    let lib = dir.path().join("lib");
    fs::create_dir(&lib).unwrap();
    fs::write(lib.join("index.ts"), "").unwrap();

    let current = dir.path().join("main.ts");
    let resolved = resolve_import(&current, "./lib").unwrap();
    assert!(resolved.ends_with("lib/index.ts"));
}

#[test]
fn test_resolve_import_explicit_extension() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("lib.ts");
    fs::write(&file, "").unwrap();

    let current = dir.path().join("main.ts");
    let resolved = resolve_import(&current, "./lib.ts").unwrap();
    assert_eq!(
        resolved.canonicalize().unwrap(),
        file.canonicalize().unwrap()
    );
}

#[test]
fn test_resolve_import_skips_non_javascript_file() {
    let dir = tempdir().unwrap();
    let stylesheet = dir.path().join("styles.css");
    fs::write(&stylesheet, "body { }").unwrap();

    let current = dir.path().join("main.ts");
    assert_eq!(resolve_import(&current, "./styles"), None);
}

#[test]
fn test_resolve_import_skips_existing_non_javascript_file_without_extension() {
    let dir = tempdir().unwrap();
    let non_js = dir.path().join("legacy");
    fs::write(&non_js, "legacy").unwrap();
    let current = dir.path().join("main.ts");

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
