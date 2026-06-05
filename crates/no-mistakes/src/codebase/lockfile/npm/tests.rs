use super::*;

const V2_LOCK: &str = r#"{
  "lockfileVersion": 2,
  "packages": {
    "": {
      "name": "my-app",
      "version": "1.0.0"
    },
    "node_modules/lodash": {
      "version": "4.17.21",
      "resolved": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz",
      "integrity": "sha512-abc123"
    },
    "node_modules/@scope/pkg": {
      "version": "2.0.0",
      "resolved": "https://registry.npmjs.org/@scope/pkg/-/@scope/pkg-2.0.0.tgz",
      "integrity": "sha512-scoped"
    },
    "node_modules/my-app/node_modules/lodash": {
      "version": "4.17.20",
      "resolved": "https://registry.npmjs.org/lodash/-/lodash-4.17.20.tgz",
      "integrity": "sha512-old"
    },
    "node_modules/workspace-pkg": {
      "version": "1.0.0",
      "link": true
    },
    "node_modules/no-resolved": {
      "version": "1.0.0"
    }
  }
}"#;

const V1_LOCK: &str = r#"{
  "lockfileVersion": 1,
  "dependencies": {
    "lodash": {
      "version": "4.17.21",
      "resolved": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz",
      "integrity": "sha512-abc123",
      "dependencies": {
        "nested-dep": {
          "version": "1.0.0",
          "resolved": "https://registry.npmjs.org/nested-dep/-/nested-dep-1.0.0.tgz",
          "integrity": "sha512-nested"
        }
      }
    }
  }
}"#;

#[test]
fn parse_v2_basic() {
    let pkgs = parse(V2_LOCK);
    let find = |name: &str| pkgs.iter().find(|p| p.name == name);

    let lodash = find("lodash").unwrap();
    assert_eq!(lodash.version, "4.17.21");
    assert_eq!(lodash.fingerprint, "sha512-abc123");
    assert_eq!(lodash.kind, ResolutionKind::Registry);
}

#[test]
fn parse_v2_scoped() {
    let pkgs = parse(V2_LOCK);
    let scoped = pkgs.iter().find(|p| p.name == "@scope/pkg").unwrap();
    assert_eq!(scoped.version, "2.0.0");
    assert_eq!(scoped.kind, ResolutionKind::Registry);
}

#[test]
fn parse_v2_nested_node_modules() {
    let pkgs = parse(V2_LOCK);
    let matches: Vec<_> = pkgs.iter().filter(|p| p.name == "lodash").collect();
    assert_eq!(matches.len(), 2);
}

#[test]
fn parse_v2_workspace_link() {
    let pkgs = parse(V2_LOCK);
    let ws = pkgs.iter().find(|p| p.name == "workspace-pkg").unwrap();
    assert_eq!(ws.kind, ResolutionKind::Workspace);
}

#[test]
fn parse_v2_no_resolved_is_directory() {
    let pkgs = parse(V2_LOCK);
    let nr = pkgs.iter().find(|p| p.name == "no-resolved").unwrap();
    assert_eq!(nr.kind, ResolutionKind::Directory);
}

#[test]
fn parse_v2_skips_root_entry() {
    let pkgs = parse(V2_LOCK);
    assert!(pkgs.iter().all(|p| !p.name.is_empty()));
}

#[test]
fn parse_v1_basic() {
    let pkgs = parse(V1_LOCK);
    let lodash = pkgs.iter().find(|p| p.name == "lodash").unwrap();
    assert_eq!(lodash.version, "4.17.21");
    assert_eq!(lodash.fingerprint, "sha512-abc123");
}

#[test]
fn parse_v1_nested_deps() {
    let pkgs = parse(V1_LOCK);
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
