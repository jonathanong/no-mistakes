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
