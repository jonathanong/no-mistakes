use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/lockfile/npm")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e))
}

#[test]
fn parse_v2_basic() {
    let lock = fixture("v2.json");
    let pkgs = parse(&lock);
    let find = |name: &str| pkgs.iter().find(|p| p.name == name);

    let lodash = find("lodash").unwrap();
    assert_eq!(lodash.version, "4.17.21");
    assert_eq!(lodash.fingerprint, "sha512-abc123");
    assert_eq!(lodash.kind, ResolutionKind::Registry);
}

#[test]
fn parse_v2_scoped() {
    let lock = fixture("v2.json");
    let pkgs = parse(&lock);
    let scoped = pkgs.iter().find(|p| p.name == "@scope/pkg").unwrap();
    assert_eq!(scoped.version, "2.0.0");
    assert_eq!(scoped.kind, ResolutionKind::Registry);
}

#[test]
fn parse_v2_nested_node_modules() {
    let lock = fixture("v2.json");
    let pkgs = parse(&lock);
    let matches: Vec<_> = pkgs.iter().filter(|p| p.name == "lodash").collect();
    assert_eq!(matches.len(), 2);
}

#[test]
fn parse_v2_workspace_link() {
    let lock = fixture("v2.json");
    let pkgs = parse(&lock);
    let ws = pkgs.iter().find(|p| p.name == "workspace-pkg").unwrap();
    assert_eq!(ws.kind, ResolutionKind::Workspace);
}

#[test]
fn parse_v2_no_resolved_is_directory() {
    let lock = fixture("v2.json");
    let pkgs = parse(&lock);
    let nr = pkgs.iter().find(|p| p.name == "no-resolved").unwrap();
    assert_eq!(nr.kind, ResolutionKind::Directory);
}

#[test]
fn parse_v2_skips_root_entry() {
    let lock = fixture("v2.json");
    let pkgs = parse(&lock);
    assert!(pkgs.iter().all(|p| !p.name.is_empty()));
}

#[test]
fn parse_v2_workspace_path_uses_name_field() {
    // packages/workspace-pkg has a "name" field → should use that, not the path key
    let lock = fixture("v2.json");
    let pkgs = parse(&lock);
    let ws_entries: Vec<_> = pkgs.iter().filter(|p| p.name == "workspace-pkg").collect();
    assert!(
        !ws_entries.is_empty(),
        "workspace-pkg should appear via name field"
    );
}

#[test]
fn parse_v1_basic() {
    let lock = fixture("v1.json");
    let pkgs = parse(&lock);
    let lodash = pkgs.iter().find(|p| p.name == "lodash").unwrap();
    assert_eq!(lodash.version, "4.17.21");
    assert_eq!(lodash.fingerprint, "sha512-abc123");
}

#[test]
fn parse_v1_nested_deps() {
    let lock = fixture("v1.json");
    let pkgs = parse(&lock);
    assert!(pkgs.iter().any(|p| p.name == "nested-dep"));
}

#[test]
fn parse_v3_uses_v2_path() {
    let lock = r#"{
      "lockfileVersion": 3,
      "packages": {
        "node_modules/react": {
          "version": "18.0.0",
          "resolved": "https://registry.npmjs.org/react/-/react-18.0.0.tgz",
          "integrity": "sha512-react"
        }
      }
    }"#;
    let pkgs = parse(lock);
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "react");
}

#[test]
fn parse_invalid_json() {
    assert!(parse("{ invalid json").is_empty());
}

#[test]
fn parse_no_packages_v2() {
    let lock = r#"{"lockfileVersion": 2}"#;
    assert!(parse(lock).is_empty());
}

#[test]
fn parse_no_dependencies_v1() {
    let lock = r#"{"lockfileVersion": 1}"#;
    assert!(parse(lock).is_empty());
}

#[test]
fn parse_v1_no_integrity_uses_resolved() {
    let lock = r#"{
      "lockfileVersion": 1,
      "dependencies": {
        "some-pkg": {
          "version": "1.0.0",
          "resolved": "https://example.com/pkg.tgz"
        }
      }
    }"#;
    let pkgs = parse(lock);
    assert_eq!(pkgs[0].fingerprint, "https://example.com/pkg.tgz");
}
