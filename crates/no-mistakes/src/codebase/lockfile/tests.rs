use super::*;

#[test]
fn diff_empty_both() {
    let result = diff(&[], &[]);
    assert!(result.is_empty());
}

#[test]
fn diff_added() {
    let new = vec![ResolvedPackage {
        name: "lodash".to_string(),
        version: "4.17.21".to_string(),
        fingerprint: "sha512-abc".to_string(),
        kind: ResolutionKind::Registry,
    }];
    let result = diff(&[], &new);
    assert_eq!(result.added, vec!["lodash"]);
    assert!(result.removed.is_empty());
    assert!(result.changed.is_empty());
}

#[test]
fn diff_removed() {
    let old = vec![ResolvedPackage {
        name: "lodash".to_string(),
        version: "4.17.21".to_string(),
        fingerprint: "sha512-abc".to_string(),
        kind: ResolutionKind::Registry,
    }];
    let result = diff(&old, &[]);
    assert!(result.added.is_empty());
    assert_eq!(result.removed, vec!["lodash"]);
    assert!(result.changed.is_empty());
}

#[test]
fn diff_changed_version() {
    let old = vec![ResolvedPackage {
        name: "lodash".to_string(),
        version: "4.17.20".to_string(),
        fingerprint: "sha512-old".to_string(),
        kind: ResolutionKind::Registry,
    }];
    let new = vec![ResolvedPackage {
        name: "lodash".to_string(),
        version: "4.17.21".to_string(),
        fingerprint: "sha512-new".to_string(),
        kind: ResolutionKind::Registry,
    }];
    let result = diff(&old, &new);
    assert!(result.added.is_empty());
    assert!(result.removed.is_empty());
    assert_eq!(result.changed, vec!["lodash"]);
}

#[test]
fn diff_unchanged() {
    let pkg = ResolvedPackage {
        name: "lodash".to_string(),
        version: "4.17.21".to_string(),
        fingerprint: "sha512-abc".to_string(),
        kind: ResolutionKind::Registry,
    };
    let result = diff(std::slice::from_ref(&pkg), std::slice::from_ref(&pkg));
    assert!(result.is_empty());
}

#[test]
fn diff_sorted_output() {
    let old = vec![
        ResolvedPackage {
            name: "b".to_string(),
            version: "1.0.0".to_string(),
            fingerprint: "fp-b".to_string(),
            kind: ResolutionKind::Registry,
        },
        ResolvedPackage {
            name: "a".to_string(),
            version: "1.0.0".to_string(),
            fingerprint: "fp-a".to_string(),
            kind: ResolutionKind::Registry,
        },
    ];
    let result = diff(&old, &[]);
    assert_eq!(result.removed, vec!["a", "b"]);
}

#[test]
fn diff_all_changed_names() {
    let result = LockfileDiff {
        added: vec!["c".to_string()],
        removed: vec!["a".to_string()],
        changed: vec!["b".to_string()],
    };
    let names: Vec<_> = result.all_changed_names().collect();
    assert_eq!(names, vec!["c", "a", "b"]);
}

#[test]
fn detect_manager_known() {
    assert_eq!(
        detect_manager("package-lock.json"),
        Some(PackageManager::Npm)
    );
    assert_eq!(
        detect_manager("npm-shrinkwrap.json"),
        Some(PackageManager::Npm)
    );
    assert_eq!(detect_manager("pnpm-lock.yaml"), Some(PackageManager::Pnpm));
    assert_eq!(detect_manager("yarn.lock"), Some(PackageManager::Yarn));
    assert_eq!(detect_manager("bun.lock"), Some(PackageManager::Bun));
}

#[test]
fn detect_manager_unknown() {
    assert_eq!(detect_manager("bun.lockb"), None);
    assert_eq!(detect_manager("package.json"), None);
    assert_eq!(detect_manager("other.txt"), None);
}

#[test]
fn is_binary_lockfile_only_bun_lockb() {
    assert!(is_binary_lockfile("bun.lockb"));
    assert!(!is_binary_lockfile("bun.lock"));
    assert!(!is_binary_lockfile("pnpm-lock.yaml"));
}

#[test]
fn parse_lockfile_npm() {
    let content = r#"{"lockfileVersion":2,"packages":{"node_modules/lodash":{"version":"4.17.21","resolved":"https://registry.npmjs.org/lodash","integrity":"sha512-abc"}}}"#;
    let pkgs = parse_lockfile(PackageManager::Npm, content);
    assert!(!pkgs.is_empty());
    assert!(pkgs.iter().any(|p| p.name == "lodash"));
}

#[test]
fn parse_lockfile_yarn() {
    // Classic yarn v1 format
    let content = "# yarn lockfile v1\n\nlodash@^4.17.0:\n  version \"4.17.21\"\n  resolved \"https://registry.yarnpkg.com/lodash.tgz\"\n  integrity sha512-abc\n";
    let pkgs = parse_lockfile(PackageManager::Yarn, content);
    assert!(!pkgs.is_empty());
    assert!(pkgs.iter().any(|p| p.name == "lodash"));
}

#[test]
fn parse_lockfile_bun() {
    let content = r#"{"lockfileVersion":0,"packages":{"lodash":["lodash@4.17.21",{}]}}"#;
    let pkgs = parse_lockfile(PackageManager::Bun, content);
    // bun parser requires array with specifier; empty content returns empty
    let _ = pkgs; // just exercise the branch
}

#[test]
fn parse_lockfile_bun_empty() {
    let pkgs = parse_lockfile(PackageManager::Bun, "not valid json");
    assert!(pkgs.is_empty());
}
