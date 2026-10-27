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
#[path = "tests/specifiers.rs"]
mod specifiers;

// Finding cases moved to cli_pkgcfg_snapshot_package_json_registry_only.

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
