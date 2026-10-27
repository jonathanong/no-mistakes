use super::tests_support::check_file;
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
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

fn fixture(path: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/no-empty-or-comments-only-files/fixture")
            .join(path),
    )
}

fn case_root(name: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("no-empty-or-comments-only-files")
            .join("cases")
            .join(name),
    )
}

#[test]
fn pass_fixture_has_no_findings() {
    let root = fixture("pass");
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
    let root = fixture("fail");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        !findings.is_empty(),
        "expected findings for comments-only file"
    );
}

#[test]
fn sort_fixture_findings_are_sorted() {
    let root = fixture("sort");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    let files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    assert_eq!(files, vec!["a.ts", "b.ts"]);
}

#[test]
fn empty_file_detected() {
    let root = case_root("empty-file-detected");
    let path = root.join("empty.ts");
    let findings = check_file(&path, &root);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("empty"));
}

#[test]
fn whitespace_only_file_detected() {
    let root = case_root("whitespace-only-file-detected");
    let path = root.join("ws.ts");
    let findings = check_file(&path, &root);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("empty"));
}

#[test]
fn js_line_comment_only() {
    let root = case_root("js-line-comment-only");
    let path = root.join("a.ts");
    let findings = check_file(&path, &root);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("only comments"));
}

#[test]
fn js_block_comment_only() {
    let root = case_root("js-block-comment-only");
    let path = root.join("a.ts");
    let findings = check_file(&path, &root);
    assert_eq!(findings.len(), 1);
}

#[test]
fn js_real_content_passes() {
    let root = case_root("js-real-content-passes");
    let path = root.join("a.ts");
    let findings = check_file(&path, &root);
    assert!(findings.is_empty());
}

#[test]
fn sql_dash_comment_only() {
    let root = case_root("sql-dash-comment-only");
    let path = root.join("query.sql");
    let findings = check_file(&path, &root);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("only comments"));
}

#[test]
fn sql_real_content_passes() {
    let root = case_root("sql-real-content-passes");
    let path = root.join("query.sql");
    assert!(check_file(&path, &root).is_empty());
}

#[test]
fn html_comment_only() {
    let root = case_root("html-comment-only");
    let path = root.join("doc.md");
    let findings = check_file(&path, &root);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("only comments"));
}

#[test]
fn html_real_content_passes() {
    let root = case_root("html-real-content-passes");
    let path = root.join("doc.md");
    assert!(check_file(&path, &root).is_empty());
}

#[test]
fn intentionally_empty_path_is_exempt() {
    let root = case_root("intentionally-empty-path-is-exempt");
    let config = config_with_rule("{extensions: [\".ts\"], intentionallyEmpty: [\"empty.ts\"]}");
    let findings = check(&root, &config).unwrap();
    assert!(findings.is_empty(), "exempt path should be skipped");
}

#[test]
fn unreadable_file_reports_unanalyzable_input() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("missing.ts");
    let findings = check_file(&path, tmp.path());
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable"));
}

#[test]
fn check_with_files_works() {
    let root = case_root("check-with-files-works");
    let path = root.join("only_comment.ts");
    let config = config_with_rule("{extensions: [\".ts\"]}");
    let findings = check_with_files(&root, &config, &[path]).unwrap();
    assert_eq!(findings.len(), 1);
}

#[test]
fn default_extensions_used_when_extensions_empty() {
    // When no extensions are configured, effective_extensions returns DEFAULT_EXTENSIONS.
    // A .ts file with only a comment should still be flagged.
    let root = case_root("default-extensions-used-when-extensions-empty");
    let path = root.join("only_comment.ts");
    let config = config_with_rule("{}"); // empty options → default extensions
    let findings = check_with_files(&root, &config, &[path]).unwrap();
    assert_eq!(
        findings.len(),
        1,
        "default extensions should catch .ts files"
    );
}

#[test]
fn unknown_extension_treated_as_raw_content() {
    let root = case_root("unknown-extension-treated-as-raw-content");
    let content_path = root.join("data.xml");
    assert!(
        check_file(&content_path, &root).is_empty(),
        "unknown ext should pass"
    );
    let empty_path = root.join("empty.xml");
    let findings = check_file(&empty_path, &root);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("empty"));
}

#[test]
fn mjs_line_comment_only_flagged() {
    let root = case_root("mjs-line-comment-only-flagged");
    let path = root.join("a.mjs");
    let findings = check_file(&path, &root);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("only comments"));
}

#[test]
fn md_html_comment_only_flagged() {
    let root = case_root("md-html-comment-only-flagged");
    let path = root.join("doc.md");
    let findings = check_file(&path, &root);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("only comments"));
}

#[test]
fn unterminated_block_comment_is_comments_only() {
    let root = case_root("unterminated-block-comment-is-comments-only");
    let path = root.join("a.ts");
    let findings = check_file(&path, &root);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("only comments"));
}
