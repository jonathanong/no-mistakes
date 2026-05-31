use super::*;
use crate::codebase::rules::{path_filter, rust_max_lines_per_file, sort_findings, target_roots};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

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

fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> anyhow::Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = rule.rule_options();
        let roots = normalize_roots(&opts, root, &target_roots(root, config, rule));
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| {
                roots.iter().any(|rule_root| path.starts_with(rule_root))
                    && !is_excluded(root, path, &opts.excludes)
                    && !rust_max_lines_per_file::is_test_file(root, path)
            })
            .cloned()
            .collect();
        let files = path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan(root, &files)?);
    }
    sort_findings(&mut findings);
    Ok(findings)
}

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/rust-no-inline-allows/fixture")
        .join(path)
}

fn check_fixture(path: &str) -> Vec<RuleFinding> {
    let path = fixture(path);
    check_file(&path, path.parent().unwrap())
}

#[test]
fn no_match_on_clean_source() {
    assert!(check_fixture("unit/clean.rs").is_empty());
}

#[test]
fn matches_inline_allow_attribute() {
    let findings = check_fixture("unit/inline_allow.rs");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 1);
    assert!(findings[0].message.contains("allow(dead_code)"));
}

#[test]
fn matches_indented_inline_allow_attribute() {
    let findings = check_fixture("unit/indented_allow.rs");
    assert_eq!(findings.len(), 1);
}

#[test]
fn matches_path_form_allow_attribute() {
    let findings = check_fixture("unit/path_allow.rs");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("allow()"));
}

#[test]
fn normalizes_namespaced_lint_tokens() {
    let findings = check_fixture("unit/namespaced_allow.rs");
    assert_eq!(findings.len(), 1);
    assert!(findings[0]
        .message
        .contains("allow(clippy::all,unused_imports)"));
}

#[test]
fn invalid_rust_source_returns_no_findings() {
    assert!(check_fixture("unit/invalid.rs").is_empty());
}

#[test]
fn unreadable_file_returns_no_findings() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("missing.rs");
    assert!(check_file(&path, tmp.path()).is_empty());
}

#[test]
fn respects_disable_file_comment() {
    assert!(check_fixture("unit/disable_file.rs").is_empty());
}

#[test]
fn check_respects_excludes() {
    let root = fixture("excludes");
    let config = config_with_rule("{excludes: [\"generated\"]}");
    let findings = check(&root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_with_files_respects_roots() {
    let root = fixture("roots");
    let sub = root.join("sub");
    let outside = root.join("a.rs");
    let inside = sub.join("b.rs");
    let config = config_with_rule("{roots: [\"sub\"]}");
    let findings = check_with_files(&root, &config, &[outside, inside]).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "sub/b.rs");
}

#[test]
fn check_with_files_respects_absolute_roots() {
    let root = fixture("roots");
    let sub = root.join("sub");
    let outside = root.join("a.rs");
    let inside = sub.join("b.rs");
    let config = config_with_rule(&format!("{{roots: [\"{}\"]}}", sub.display()));
    let findings = check_with_files(&root, &config, &[outside, inside]).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "sub/b.rs");
}
