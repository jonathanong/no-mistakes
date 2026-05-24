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
            .join("../../fixtures/rules/no-empty-or-comments-only-files")
            .join(path),
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
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty.ts");
    std::fs::write(&path, "").unwrap();
    let findings = check_file(&path, tmp.path());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("empty"));
}

#[test]
fn whitespace_only_file_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("ws.ts");
    std::fs::write(&path, "   \n   \n").unwrap();
    let findings = check_file(&path, tmp.path());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("empty"));
}

#[test]
fn js_line_comment_only() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.ts");
    std::fs::write(&path, "// TODO: implement later\n").unwrap();
    let findings = check_file(&path, tmp.path());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("only comments"));
}

#[test]
fn js_block_comment_only() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.ts");
    std::fs::write(&path, "/* block comment */\n").unwrap();
    let findings = check_file(&path, tmp.path());
    assert_eq!(findings.len(), 1);
}

#[test]
fn js_real_content_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.ts");
    std::fs::write(&path, "// comment\nexport const x = 1;\n").unwrap();
    let findings = check_file(&path, tmp.path());
    assert!(findings.is_empty());
}

#[test]
fn sql_dash_comment_only() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("query.sql");
    std::fs::write(&path, "-- drop table\n").unwrap();
    let findings = check_file(&path, tmp.path());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("only comments"));
}

#[test]
fn sql_real_content_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("query.sql");
    std::fs::write(&path, "SELECT 1;\n").unwrap();
    assert!(check_file(&path, tmp.path()).is_empty());
}

#[test]
fn html_comment_only() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("doc.md");
    std::fs::write(&path, "<!-- todo -->\n").unwrap();
    let findings = check_file(&path, tmp.path());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("only comments"));
}

#[test]
fn html_real_content_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("doc.md");
    std::fs::write(&path, "# Title\n<!-- note -->\n").unwrap();
    assert!(check_file(&path, tmp.path()).is_empty());
}

#[test]
fn intentionally_empty_path_is_exempt() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("empty.ts");
    std::fs::write(&path, "").unwrap();
    let config = config_with_rule("{extensions: [\".ts\"], intentionallyEmpty: [\"empty.ts\"]}");
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty(), "exempt path should be skipped");
}

#[test]
fn unreadable_file_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("missing.ts");
    let findings = check_file(&path, tmp.path());
    assert!(findings.is_empty());
}

#[test]
fn check_with_files_works() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("only_comment.ts");
    std::fs::write(&path, "// comment\n").unwrap();
    let config = config_with_rule("{extensions: [\".ts\"]}");
    let findings = check_with_files(tmp.path(), &config, &[path]).unwrap();
    assert_eq!(findings.len(), 1);
}

#[test]
fn default_extensions_used_when_extensions_empty() {
    // When no extensions are configured, effective_extensions returns DEFAULT_EXTENSIONS.
    // A .ts file with only a comment should still be flagged.
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("only_comment.ts");
    std::fs::write(&path, "// comment only\n").unwrap();
    let config = config_with_rule("{}"); // empty options → default extensions
    let findings = check_with_files(tmp.path(), &config, &[path]).unwrap();
    assert_eq!(
        findings.len(),
        1,
        "default extensions should catch .ts files"
    );
}

#[test]
fn unknown_extension_treated_as_raw_content() {
    let tmp = tempfile::tempdir().unwrap();
    let content_path = tmp.path().join("data.xml");
    std::fs::write(&content_path, "some content").unwrap();
    assert!(
        check_file(&content_path, tmp.path()).is_empty(),
        "unknown ext should pass"
    );
    let empty_path = tmp.path().join("empty.xml");
    std::fs::write(&empty_path, "   ").unwrap();
    let findings = check_file(&empty_path, tmp.path());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("empty"));
}

#[test]
fn mjs_line_comment_only_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.mjs");
    std::fs::write(&path, "// comment\n").unwrap();
    let findings = check_file(&path, tmp.path());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("only comments"));
}

#[test]
fn md_html_comment_only_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("doc.md");
    std::fs::write(&path, "<!-- only a comment -->\n").unwrap();
    let findings = check_file(&path, tmp.path());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("only comments"));
}

#[test]
fn unterminated_block_comment_is_comments_only() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("a.ts");
    std::fs::write(&path, "/* unterminated block comment").unwrap();
    let findings = check_file(&path, tmp.path());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("only comments"));
}
