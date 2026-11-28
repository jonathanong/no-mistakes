use super::*;

const SAMPLE_LOCK: &str = r#"
lockfileVersion: '9.0'

packages:
  lodash@4.17.21:
    resolution: {integrity: sha512-abc123}

  github.com/org/repo:
    resolution: {repo: 'https://github.com/org/repo', commit: abc123}

  some-tarball@1.0.0:
    resolution: {tarball: 'https://example.com/pkg.tgz'}

  local-pkg@1.0.0:
    resolution: {directory: '../local-pkg'}

  no-resolution@1.0.0: {}
"#;

#[test]
fn test_parse_sample_lock() {
    let packages = parse_pnpm_lock(SAMPLE_LOCK);
    assert_eq!(packages.len(), 5);

    let find = |key: &str| {
        packages
            .iter()
            .find(|p| p.key == key)
            .map(|p| p.resolution_kind.as_str())
    };

    assert_eq!(find("lodash@4.17.21"), Some("integrity"));
    assert_eq!(find("github.com/org/repo"), Some("repo"));
    assert_eq!(find("some-tarball@1.0.0"), Some("tarball"));
    assert_eq!(find("local-pkg@1.0.0"), Some("directory"));
    assert_eq!(find("no-resolution@1.0.0"), Some(""));
}

#[test]
fn test_empty_content() {
    let packages = parse_pnpm_lock("");
    assert!(packages.is_empty());
}

#[test]
fn test_no_packages_section() {
    let content = "lockfileVersion: '9.0'\n";
    let packages = parse_pnpm_lock(content);
    assert!(packages.is_empty());
}

#[test]
fn test_invalid_yaml() {
    let packages = parse_pnpm_lock("{ invalid: yaml: content: [[[");
    assert!(packages.is_empty());
}

#[test]
fn test_repo_priority_over_commit() {
    let content = r#"
packages:
  mypkg@1.0.0:
    resolution: {repo: 'https://example.com', commit: abc, tarball: 'https://t.tgz'}
"#;
    let packages = parse_pnpm_lock(content);
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].resolution_kind, "repo");
}

#[test]
fn test_commit_only() {
    let content = r#"
packages:
  mypkg@1.0.0:
    resolution: {commit: abc123def}
"#;
    let packages = parse_pnpm_lock(content);
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].resolution_kind, "commit");
}

#[test]
fn test_integrity_only_resolution() {
    // Exercises line 52: `return "integrity".to_string()` and its closing brace (53).
    let content = r#"
packages:
  normal-pkg@1.0.0:
    resolution: {integrity: sha512-xyz}
"#;
    let packages = parse_pnpm_lock(content);
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].resolution_kind, "integrity");
}

#[test]
fn test_resolution_with_unknown_key_returns_empty() {
    // Exercises line 55: `String::new()` when resolution has no recognized keys.
    let content = r#"
packages:
  exotic@1.0.0:
    resolution: {checksum: abc123}
"#;
    let packages = parse_pnpm_lock(content);
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].resolution_kind, "");
}

#[test]
fn test_numeric_package_key() {
    // Exercises yaml_value_to_string line 61: Number branch.
    // YAML treats bare integers as numbers when used as mapping keys.
    let content = "packages:\n  1234:\n    resolution: {integrity: sha512-num}\n";
    let packages = parse_pnpm_lock(content);
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].key, "1234");
}

#[test]
fn test_bool_package_key() {
    // Exercises yaml_value_to_string line 62: Bool branch.
    let content = "packages:\n  true:\n    resolution: {integrity: sha512-bool}\n";
    let packages = parse_pnpm_lock(content);
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].key, "true");
}

#[test]
fn test_null_package_key() {
    // Exercises yaml_value_to_string line 63: `_ => String::new()` for Null.
    let content = "packages:\n  ~:\n    resolution: {integrity: sha512-null}\n";
    let packages = parse_pnpm_lock(content);
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].key, "");
}
