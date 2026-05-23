use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/package-json-registry-only")
        .join(path)
}

fn config_with_options(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

// ── is_blocked_specifier tests ──────────────────────────────────────────────

#[test]
fn semver_range_is_allowed() {
    assert!(!is_blocked_specifier("^4.17.21"));
}

#[test]
fn workspace_is_allowed() {
    assert!(!is_blocked_specifier("workspace:*"));
}

#[test]
fn catalog_is_allowed() {
    assert!(!is_blocked_specifier("catalog:"));
}

#[test]
fn npm_alias_allowed() {
    assert!(!is_blocked_specifier("npm:lodash@^4"));
}

#[test]
fn file_specifier_blocked() {
    assert!(is_blocked_specifier("file:../my-lib"));
}

#[test]
fn link_specifier_blocked() {
    assert!(is_blocked_specifier("link:../other"));
}

#[test]
fn portal_specifier_blocked() {
    assert!(is_blocked_specifier("portal:../pkg"));
}

#[test]
fn patch_specifier_blocked() {
    assert!(is_blocked_specifier("patch:lodash@4.17.21"));
}

#[test]
fn git_specifier_blocked() {
    assert!(is_blocked_specifier("git+https://github.com/foo/bar.git"));
}

#[test]
fn github_shorthand_blocked() {
    assert!(is_blocked_specifier("github:owner/repo"));
}

#[test]
fn owner_slash_repo_shorthand_blocked() {
    assert!(is_blocked_specifier("owner/repo"));
}

#[test]
fn scoped_package_allowed() {
    assert!(!is_blocked_specifier("@scope/pkg"));
}

#[test]
fn scoped_package_with_extra_slash_blocked() {
    assert!(is_blocked_specifier("@scope/pkg/sub"));
}

#[test]
fn http_url_blocked() {
    assert!(is_blocked_specifier("http://example.com/pkg.tgz"));
}

#[test]
fn https_url_blocked() {
    assert!(is_blocked_specifier("https://example.com/pkg.tgz"));
}

// ── fixture tests ────────────────────────────────────────────────────────────

#[test]
fn pass_fixture_produces_no_findings() {
    let root = fixture("pass");
    let config = config_with_options("{}");
    let files = vec![root.join("package.json")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn fail_fixture_produces_findings() {
    let root = fixture("fail");
    let config = config_with_options("{}");
    let files = vec![root.join("package.json")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(!findings.is_empty(), "expected at least one finding");
    assert!(
        findings.iter().any(|f| f.message.contains("file:")),
        "expected file: finding"
    );
}

// ── inline check_package_json tests ─────────────────────────────────────────

fn write_package_json(dir: &Path, content: &str) -> PathBuf {
    let p = dir.join("package.json");
    std::fs::write(&p, content).unwrap();
    p
}

#[test]
fn all_dep_fields_checked() {
    let tmp = tempfile::tempdir().unwrap();
    let content = r#"{
        "dependencies": {"a": "file:../a"},
        "devDependencies": {"b": "link:../b"},
        "peerDependencies": {"c": "git+https://github.com/x/y"},
        "optionalDependencies": {"d": "^1.0.0"}
    }"#;
    let path = write_package_json(tmp.path(), content);
    let findings = check_package_json(&path, tmp.path());
    assert_eq!(findings.len(), 3);
}

#[test]
fn node_modules_excluded() {
    let tmp = tempfile::tempdir().unwrap();
    let nm = tmp.path().join("node_modules").join("some-pkg");
    std::fs::create_dir_all(&nm).unwrap();
    write_package_json(&nm, r#"{"dependencies": {"x": "file:../x"}}"#);
    let config = config_with_options("{}");
    let files = vec![nm.join("package.json")];
    let findings = check_with_files(tmp.path(), &config, &files).unwrap();
    assert!(findings.is_empty(), "node_modules must be excluded");
}

#[test]
fn unreadable_package_json_returns_empty() {
    let findings = check_package_json(Path::new("/nonexistent/package.json"), Path::new("/"));
    assert!(findings.is_empty());
}

#[test]
fn finding_message_format() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_package_json(
        tmp.path(),
        r#"{"dependencies": {"my-lib": "file:../my-lib"}}"#,
    );
    let findings = check_package_json(&path, tmp.path());
    assert_eq!(findings.len(), 1);
    assert!(findings[0]
        .message
        .contains("\"my-lib\": \"file:../my-lib\""));
    assert!(findings[0].message.contains("not allowed"));
}

#[test]
fn lockfile_non_registry_resolution_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let lockfile_content =
        "packages:\n  my-pkg@1.0.0:\n    resolution:\n      tarball: https://example.com/pkg.tgz\n";
    std::fs::write(tmp.path().join("pnpm-lock.yaml"), lockfile_content).unwrap();
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(tmp.path(), &config, &[]).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("tarball"));
}

#[test]
fn lockfile_registry_integrity_allowed() {
    let tmp = tempfile::tempdir().unwrap();
    let lockfile_content =
        "packages:\n  lodash@4.17.21:\n    resolution:\n      integrity: sha512-abc\n";
    std::fs::write(tmp.path().join("pnpm-lock.yaml"), lockfile_content).unwrap();
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(tmp.path(), &config, &[]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn missing_lockfile_is_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_with_options("lockfile: nonexistent.yaml");
    let findings = check_with_files(tmp.path(), &config, &[]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn scopes_filter_package_json_files() {
    let tmp = tempfile::tempdir().unwrap();
    let included = tmp.path().join("packages/a");
    let excluded = tmp.path().join("packages/b");
    std::fs::create_dir_all(&included).unwrap();
    std::fs::create_dir_all(&excluded).unwrap();
    write_package_json(&included, r#"{"dependencies": {"bad": "file:../bad"}}"#);
    write_package_json(
        &excluded,
        r#"{"dependencies": {"also-bad": "file:../bad"}}"#,
    );
    let config = config_with_options("scopes:\n  - packages/a\n");
    let files = vec![included.join("package.json"), excluded.join("package.json")];
    let findings = check_with_files(tmp.path(), &config, &files).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].file.contains("packages/a"));
}
