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
            .join("../../test-cases/rules/file-extension-policy/fixture")
            .join(subpath),
    )
}

fn case_root(name: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("file-extension-policy")
            .join("cases")
            .join(name),
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
    let root = case_root("banned-extension-in-scope-flagged");
    let path = root.join("backend/index.js");
    let allowlist = HashSet::new();
    let scopes = vec![make_scope("backend", &[".js", ".ts"])];
    let findings = check_file(&path, &root, &allowlist, &scopes);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains(".js"));
}

#[test]
fn allowed_extension_passes() {
    let root = case_root("allowed-extension-passes");
    let path = root.join("backend/index.mts");
    let allowlist = HashSet::new();
    let scopes = vec![make_scope("backend", &[".js", ".ts"])];
    let findings = check_file(&path, &root, &allowlist, &scopes);
    assert!(findings.is_empty());
}

#[test]
fn declaration_file_skipped() {
    let root = case_root("declaration-file-skipped");
    let path = root.join("backend/types.d.ts");
    let allowlist = HashSet::new();
    let scopes = vec![make_scope("backend", &[".ts"])];
    let findings = check_file(&path, &root, &allowlist, &scopes);
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
    let root = case_root("allowlisted-path-skipped");
    let path = root.join("backend/legacy.js");
    let mut allowlist = HashSet::new();
    allowlist.insert("backend/legacy.js");
    let scopes = vec![make_scope("backend", &[".js"])];
    let findings = check_file(&path, &root, &allowlist, &scopes);
    assert!(findings.is_empty(), "allowlisted path should be skipped");
}

#[test]
fn out_of_scope_path_not_flagged() {
    let root = case_root("out-of-scope-path-not-flagged");
    let path = root.join("scripts/build.js");
    let allowlist = HashSet::new();
    let scopes = vec![make_scope("backend", &[".js"])];
    let findings = check_file(&path, &root, &allowlist, &scopes);
    assert!(
        findings.is_empty(),
        "out-of-scope path should not be flagged"
    );
}

#[test]
fn check_with_files_works() {
    let root = case_root("check-with-files-works");
    let path = root.join("backend/index.ts");
    let config = config_with_rule("scopes:\n  - path: backend\n    bannedExtensions: [\".ts\"]");
    let findings = check_with_files(&root, &config, &[path]).unwrap();
    assert_eq!(findings.len(), 1);
}

#[test]
fn file_with_no_extension_has_empty_ext_and_is_not_flagged() {
    // file_extension returns "" when there is no dot, and banned_extensions
    // won't match "", so no finding is emitted.
    let root = case_root("file-with-no-extension-has-empty-ext-and-is-not-flagged");
    let path = root.join("backend/Makefile"); // no dot
    let allowlist = HashSet::new();
    let scopes = vec![make_scope("backend", &[".js", ".ts"])];
    let findings = check_file(&path, &root, &allowlist, &scopes);
    assert!(
        findings.is_empty(),
        "file with no extension should not match any banned extension"
    );
}

#[test]
fn dot_prefix_scope_fixture_flags_helper_js() {
    let root = fixture_root("dot-prefix-fail");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.file == "src/helper.js"),
        "{findings:?}"
    );
}

#[test]
fn normalized_scope_prefixes_match_src_helper() {
    // `src/../src` is the same prefix as `src`. Do not compare the raw string.
    let root = fixture_root("dot-prefix-fail");
    let path = root.join("src/helper.js");
    let allowlist = HashSet::new();
    for scope in ["src", "./src", "src/../src", "src/"] {
        let findings = check_file(&path, &root, &allowlist, &[make_scope(scope, &[".js"])]);
        assert_eq!(findings.len(), 1, "{scope}: {findings:?}");
        assert_eq!(findings[0].file, "src/helper.js");
    }
}

#[test]
fn escaping_scope_is_one_config_finding() {
    let root = fixture_root("scope-escape");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert_eq!(findings[0].line, 1);
    assert!(
        findings[0]
            .message
            .contains("../outside: configured root escapes the repository"),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.file != "src/in-repo.js"),
        "{findings:?}"
    );
}

#[test]
fn scope_that_escapes_the_repo_matches_nothing() {
    let root = fixture_root("dot-prefix-fail");
    let path = root.join("src/helper.js");
    let allowlist = HashSet::new();
    for scope in ["..", "../src", "src/../..", "foo/../../src"] {
        let findings = check_file(&path, &root, &allowlist, &[make_scope(scope, &[".js"])]);
        assert!(
            findings.is_empty(),
            "{scope} matched inside the repo: {findings:?}"
        );
    }
}
