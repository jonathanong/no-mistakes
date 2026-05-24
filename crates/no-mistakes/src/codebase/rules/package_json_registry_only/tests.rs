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
fn npm_alias_scoped_package_blocked() {
    // npm:@scope/pkg@version — exercises lines 78-79 (scoped npm: alias path).
    assert!(is_blocked_specifier("npm:@scope/pkg@github:owner/repo"));
}

#[test]
fn npm_alias_scoped_package_without_version_is_safe() {
    // npm:@scope/pkg with no version — after_at.find('@') = None → version = "" → false.
    assert!(!is_blocked_specifier("npm:@scope/pkg"));
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

#[test]
fn check_standalone_produces_no_findings_for_empty_dir() {
    // Exercises the check() fn (lines 43-51) via discover_files on empty dir.
    let tmp = tempfile::tempdir().unwrap();
    let config = config_with_options("{}");
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn invalid_json_in_package_json_is_skipped() {
    // check_package_json returns Vec::new() on JSON parse error (line 120).
    let tmp = tempfile::tempdir().unwrap();
    let path = write_package_json(tmp.path(), "not json at all {{{");
    let findings = check_package_json(&path, tmp.path());
    assert!(
        findings.is_empty(),
        "invalid JSON should produce no findings"
    );
}

#[test]
fn absolute_scope_path_is_supported() {
    // The scope path is absolute, so the s.clone() branch (line 102) is taken.
    let tmp = tempfile::tempdir().unwrap();
    let pkg_dir = tmp.path().join("a");
    std::fs::create_dir_all(&pkg_dir).unwrap();
    write_package_json(&pkg_dir, r#"{"dependencies": {"bad": "file:../bad"}}"#);
    // Use the absolute path of the package dir as a scope
    let abs_scope = pkg_dir.to_string_lossy().to_string();
    let config = config_with_options(&format!("scopes:\n  - {abs_scope}\n"));
    let files = vec![pkg_dir.join("package.json")];
    let findings = check_with_files(tmp.path(), &config, &files).unwrap();
    assert_eq!(
        findings.len(),
        1,
        "absolute scope should include the package"
    );
}

#[test]
fn lockfile_with_invalid_yaml_is_skipped() {
    // check_lockfile returns Vec::new() on YAML parse error (line 163).
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("pnpm-lock.yaml"), ": invalid: yaml: {{{").unwrap();
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(tmp.path(), &config, &[]).unwrap();
    assert!(
        findings.is_empty(),
        "invalid YAML lockfile should be skipped"
    );
}

#[test]
fn lockfile_yaml_without_packages_key_is_skipped() {
    // yaml.get("packages") returns None → early return (line 167).
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("pnpm-lock.yaml"),
        "lockfileVersion: '9.0'\n",
    )
    .unwrap();
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(tmp.path(), &config, &[]).unwrap();
    assert!(
        findings.is_empty(),
        "lockfile without 'packages' key should produce no findings"
    );
}

#[test]
fn lockfile_package_without_resolution_is_skipped() {
    // Package entry exists but has no "resolution" key → continue (line 178).
    let tmp = tempfile::tempdir().unwrap();
    let lockfile = "packages:\n  my-pkg@1.0.0:\n    engines:\n      node: '>=18'\n";
    std::fs::write(tmp.path().join("pnpm-lock.yaml"), lockfile).unwrap();
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(tmp.path(), &config, &[]).unwrap();
    assert!(
        findings.is_empty(),
        "package with no resolution field should not produce a finding"
    );
}

#[test]
fn absolute_lockfile_path_is_resolved_correctly() {
    // The lockfile_path.is_absolute() branch (line 155).
    let tmp = tempfile::tempdir().unwrap();
    let lockfile = tmp.path().join("pnpm-lock.yaml");
    let lockfile_content =
        "packages:\n  my-pkg@1.0.0:\n    resolution:\n      integrity: sha512-abc\n";
    std::fs::write(&lockfile, lockfile_content).unwrap();
    // Pass the absolute path as the lockfile option
    let abs_path = lockfile.to_string_lossy().to_string();
    let config = config_with_options(&format!("lockfile: {abs_path}"));
    let findings = check_with_files(tmp.path(), &config, &[]).unwrap();
    assert!(
        findings.is_empty(),
        "registry-only lockfile with absolute path should pass"
    );
}
