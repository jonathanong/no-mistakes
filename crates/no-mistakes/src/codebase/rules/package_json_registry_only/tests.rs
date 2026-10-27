use super::scan::check_package_json_with_sources;
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
#[path = "tests/support.rs"]
mod support;
use support::*;
#[path = "tests/inputs.rs"]
mod inputs;
#[path = "tests/pnpm12.rs"]
mod pnpm12;
#[path = "tests/specifiers.rs"]
mod specifiers;

#[test]
fn all_dep_fields_checked() {
    let root = fixture("all-dep-fields");
    let path = root.join("package.json");
    let findings = check_package_json(&path, &root);
    assert_eq!(findings.len(), 3);
}

#[test]
fn node_modules_excluded() {
    let root = fixture("node-modules-excluded");
    let pkg = root.join("node_modules/some-pkg/package.json");
    let config = config_with_options("{}");
    let files = vec![pkg];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "node_modules must be excluded");
}

#[test]
fn finding_message_format() {
    let root = fixture("finding-message");
    let path = root.join("package.json");
    let findings = check_package_json(&path, &root);
    assert_eq!(findings.len(), 1);
    assert!(findings[0]
        .message
        .contains("\"my-lib\": \"file:../my-lib\""));
    assert!(findings[0].message.contains("not allowed"));
}

#[test]
fn lockfile_non_registry_resolution_flagged() {
    let root = fixture("lockfile-tarball");
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(&root, &config, &[]).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("tarball"));
}

#[test]
fn lockfile_path_filters_are_honored() {
    let root = fixture("lockfile-tarball");
    let mut config = config_with_options("lockfile: pnpm-lock.yaml");
    config.rules[0].exclude = vec!["pnpm-lock.yaml".to_string()];

    let findings = check_with_files(&root, &config, &[]).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn lockfile_registry_integrity_allowed() {
    let root = fixture("lockfile-integrity");
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(&root, &config, &[]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn scopes_filter_package_json_files() {
    let root = fixture("scopes-filter");
    let included = root.join("packages/a/package.json");
    let excluded = root.join("packages/b/package.json");
    let config = config_with_options("scopes:\n  - packages/a\n");
    let files = vec![included, excluded];
    let findings = check_with_files(&root, &config, &files).unwrap();
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
fn absolute_scope_path_is_supported() {
    // The scope path is absolute, so the s.clone() branch (line 102) is taken.
    let root = fixture("absolute-scope");
    let pkg_dir = root.join("a");
    // Use the absolute path of the package dir as a scope
    let abs_scope = pkg_dir.to_string_lossy().to_string();
    let config = config_with_options(&format!("scopes:\n  - {abs_scope}\n"));
    let files = vec![pkg_dir.join("package.json")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(
        findings.len(),
        1,
        "absolute scope should include the package"
    );
}

#[test]
fn lockfile_with_invalid_yaml_is_reported() {
    // A configured lockfile that cannot be parsed must not pass silently.
    let root = fixture("lockfile-invalid-yaml");
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(&root, &config, &[]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("could not be parsed"),
        "{findings:?}"
    );
}

#[test]
fn lockfile_yaml_without_packages_key_is_skipped() {
    // yaml.get("packages") returns None → early return (line 167).
    let root = fixture("lockfile-no-packages");
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(&root, &config, &[]).unwrap();
    assert!(
        findings.is_empty(),
        "lockfile without a packages mapping should produce no findings: {findings:?}"
    );
}

#[test]
fn lockfile_package_without_resolution_is_skipped() {
    // Package entry exists but has no "resolution" key → continue (line 178).
    let root = fixture("lockfile-no-resolution");
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(&root, &config, &[]).unwrap();
    assert!(
        findings.is_empty(),
        "package with no resolution field should not produce a finding"
    );
}

#[test]
fn absolute_lockfile_path_is_resolved_correctly() {
    // The lockfile_path.is_absolute() branch (line 155).
    let root = fixture("lockfile-integrity");
    let lockfile = root.join("pnpm-lock.yaml");
    // Pass the absolute path as the lockfile option
    let abs_path = lockfile.to_string_lossy().to_string();
    let config = config_with_options(&format!("lockfile: {abs_path}"));
    let findings = check_with_files(&root, &config, &[]).unwrap();
    assert!(
        findings.is_empty(),
        "registry-only lockfile with absolute path should pass"
    );
}
