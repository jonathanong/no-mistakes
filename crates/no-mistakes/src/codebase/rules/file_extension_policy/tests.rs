use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::collections::HashSet;
use std::path::Path;

fn config_with_rule(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn fixture_root(subpath: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/file-extension-policy")
            .join(subpath),
    )
}

#[test]
fn pass_fixture_has_no_findings() {
    let root = fixture_root("pass");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        findings.is_empty(),
        "expected no findings, got: {findings:?}"
    );
}

#[test]
fn fail_fixture_has_findings() {
    let root = fixture_root("fail");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        !findings.is_empty(),
        "expected findings for banned extension"
    );
}

#[test]
fn overlap_fixture_reports_one_finding() {
    let root = fixture_root("overlap");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "expected one finding, got: {findings:?}");
}

#[test]
fn prefix_fixture_has_no_findings() {
    let root = fixture_root("prefix-pass");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        findings.is_empty(),
        "backend scope should not match backend2: {findings:?}"
    );
}

fn make_scope(path: &str, exts: &[&str]) -> ScopeSpec {
    ScopeSpec {
        path: path.to_string(),
        banned_extensions: exts.iter().map(|s| s.to_string()).collect(),
    }
}

#[test]
fn banned_extension_in_scope_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend")).unwrap();
    let path = root.join("backend/index.js");
    std::fs::write(&path, "module.exports = {};\n").unwrap();
    let allowlist = HashSet::new();
    let scopes = vec![make_scope("backend", &[".js", ".ts"])];
    let findings = check_file(&path, root, &allowlist, &scopes);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains(".js"));
}

#[test]
fn allowed_extension_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend")).unwrap();
    let path = root.join("backend/index.mts");
    std::fs::write(&path, "export {};\n").unwrap();
    let allowlist = HashSet::new();
    let scopes = vec![make_scope("backend", &[".js", ".ts"])];
    let findings = check_file(&path, root, &allowlist, &scopes);
    assert!(findings.is_empty());
}

#[test]
fn declaration_file_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend")).unwrap();
    let path = root.join("backend/types.d.ts");
    std::fs::write(&path, "export type Foo = string;\n").unwrap();
    let allowlist = HashSet::new();
    let scopes = vec![make_scope("backend", &[".ts"])];
    let findings = check_file(&path, root, &allowlist, &scopes);
    assert!(findings.is_empty(), ".d.ts should be exempt");
}

#[test]
fn declaration_file_variants_fixture_has_no_findings() {
    let root = fixture_root("declarations-pass");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn allowlisted_path_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend")).unwrap();
    let path = root.join("backend/legacy.js");
    std::fs::write(&path, "// legacy\n").unwrap();
    let mut allowlist = HashSet::new();
    allowlist.insert("backend/legacy.js");
    let scopes = vec![make_scope("backend", &[".js"])];
    let findings = check_file(&path, root, &allowlist, &scopes);
    assert!(findings.is_empty(), "allowlisted path should be skipped");
}

#[test]
fn out_of_scope_path_not_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("scripts")).unwrap();
    let path = root.join("scripts/build.js");
    std::fs::write(&path, "// build\n").unwrap();
    let allowlist = HashSet::new();
    let scopes = vec![make_scope("backend", &[".js"])];
    let findings = check_file(&path, root, &allowlist, &scopes);
    assert!(
        findings.is_empty(),
        "out-of-scope path should not be flagged"
    );
}

#[test]
fn check_with_files_works() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend")).unwrap();
    let path = root.join("backend/index.ts");
    std::fs::write(&path, "export {};\n").unwrap();
    let config = config_with_rule("scopes:\n  - path: backend\n    bannedExtensions: [\".ts\"]");
    let findings = check_with_files(root, &config, &[path]).unwrap();
    assert_eq!(findings.len(), 1);
}

#[test]
fn file_with_no_extension_has_empty_ext_and_is_not_flagged() {
    // file_extension returns "" when there is no dot, and banned_extensions
    // won't match "", so no finding is emitted.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend")).unwrap();
    let path = root.join("backend/Makefile"); // no dot
    std::fs::write(&path, "all:\n\techo hi\n").unwrap();
    let allowlist = HashSet::new();
    let scopes = vec![make_scope("backend", &[".js", ".ts"])];
    let findings = check_file(&path, root, &allowlist, &scopes);
    assert!(
        findings.is_empty(),
        "file with no extension should not match any banned extension"
    );
}
