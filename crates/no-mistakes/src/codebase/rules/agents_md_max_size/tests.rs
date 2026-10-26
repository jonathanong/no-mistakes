use super::agents_md_max_size_budget::{check_content_with_deferred_suppression, count_lines};
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn check_file(
    path: &std::path::Path,
    root: &std::path::Path,
    max_lines: usize,
    max_chars: usize,
) -> Vec<RuleFinding> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    check_content_with_deferred_suppression(path, root, max_lines, max_chars, &content, false)
}

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

fn case_root(name: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("agents-md-max-size")
            .join("cases")
            .join(name),
    )
}

#[test]
fn count_lines_empty() {
    assert_eq!(count_lines(""), 0);
}

#[test]
fn count_lines_single_no_newline() {
    assert_eq!(count_lines("hello"), 1);
}

#[test]
fn count_lines_with_trailing_newline() {
    assert_eq!(count_lines("a\nb\n"), 2);
}

#[test]
fn count_lines_no_trailing_newline() {
    assert_eq!(count_lines("a\nb"), 2);
}

#[test]
fn count_lines_multibyte() {
    assert_eq!(count_lines("héllo\nwörld\n"), 2);
}

#[test]
fn check_file_passes_within_limits() {
    let root = case_root("check-file-passes-within-limits");
    let path = root.join("AGENTS.md");
    let findings = check_file(&path, &root, 10, 1000);
    assert!(findings.is_empty());
}

#[test]
fn check_file_fails_line_count() {
    let root = case_root("check-file-fails-line-count");
    let path = root.join("AGENTS.md");
    let findings = check_file(&path, &root, 2, 10000);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("3 lines"));
}

#[test]
fn check_file_fails_char_count() {
    let root = case_root("check-file-fails-char-count");
    let path = root.join("CLAUDE.md");
    let findings = check_file(&path, &root, 100, 5);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("characters"));
    assert!(findings[0].message.contains("bytes"));
    assert!(findings[0].message.contains("over"));
}

#[test]
fn check_file_fails_both() {
    let root = case_root("check-file-fails-both");
    let path = root.join("AGENTS.md");
    let findings = check_file(&path, &root, 1, 1);
    assert_eq!(findings.len(), 2);
}

#[test]
fn check_file_respects_disable_file_comment() {
    let root = case_root("check-file-respects-disable-file-comment");
    let path = root.join("AGENTS.md");
    let findings = check_file(&path, &root, 2, 10000);
    assert!(findings.is_empty());
}

#[test]
fn check_file_multibyte_chars() {
    let root = case_root("check-file-multibyte-chars");
    let path = root.join("AGENTS.md");
    let findings = check_file(&path, &root, 100, 3);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("5 characters"));
}

#[test]
fn advisories_report_near_limit_files_without_findings() {
    let root = case_root("advisories-report-near-limit");
    let path = root.join("CLAUDE.md");
    let config = config_with_rule("{maxChars: 20, advisoryCharsRemaining: 10}");
    let files = vec![path];

    let findings = check_with_files(&root, &config, &files).unwrap();
    let advisories = advisories_with_files(&root, &config, &files).unwrap();

    assert!(findings.is_empty());
    assert_eq!(advisories.len(), 1);
    assert!(advisories[0].message.contains("11 characters"));
    assert!(advisories[0].message.contains("11 bytes"));
    assert!(advisories[0].message.contains("9 remaining"));
}

#[test]
fn checks_advisories_and_suppressions_share_one_source_read() {
    let root = case_root("checks-advisories-and-suppressions-share-one-source-read");
    let path = root.join("CLAUDE.md");
    let config = config_with_rule("{maxChars: 20, advisoryCharsRemaining: 10}");
    let files = vec![path];
    let sources = super::super::source_store_for_files(&files);

    let findings = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
    let advisories = advisories_with_files_and_sources(&root, &config, &files, &sources).unwrap();

    assert!(findings.is_empty());
    assert_eq!(advisories.len(), 1);
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn prepared_scan_ignores_missing_source_files() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/agents-md-max-size/pass");
    let missing = root.join("missing/AGENTS.md");
    let files = vec![missing];
    let sources = super::super::source_store_for_files(&files);
    let findings = check_with_files_sources_and_deferred_suppression(
        &root,
        &config_with_rule("{maxChars: 20}"),
        &files,
        &sources,
        false,
    )
    .unwrap();

    assert!(findings.is_empty());
}

#[test]
fn advisories_skip_over_limit_files() {
    let root = case_root("advisories-skip-over-limit-files");
    let path = root.join("CLAUDE.md");
    let config = config_with_rule("{maxChars: 5, advisoryCharsRemaining: 10}");
    let files = vec![path];

    let findings = check_with_files(&root, &config, &files).unwrap();
    let advisories = advisories_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert!(advisories.is_empty());
}

#[test]
fn advisories_skip_files_outside_threshold() {
    let root = case_root("advisories-skip-files-outside-threshold");
    let path = root.join("CLAUDE.md");
    let config = config_with_rule("{maxChars: 20, advisoryCharsRemaining: 10}");
    let files = vec![path];

    let advisories = advisories_with_files(&root, &config, &files).unwrap();

    assert!(advisories.is_empty());
}

#[test]
fn advisories_skip_unreadable_files() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("CLAUDE.md");
    let config = config_with_rule("{maxChars: 20, advisoryCharsRemaining: 20}");
    let files = vec![path];

    let advisories = advisories_with_files(tmp.path(), &config, &files).unwrap();

    assert!(advisories.is_empty());
}

#[test]
fn advisories_respect_disable_file_comment() {
    let root = case_root("advisories-respect-disable-file-comment");
    let path = root.join("CLAUDE.md");
    let config = config_with_rule("{maxChars: 100, advisoryCharsRemaining: 100}");
    let files = vec![path];

    let advisories = advisories_with_files(&root, &config, &files).unwrap();

    assert!(advisories.is_empty());
}

#[test]
fn advisories_respect_line_suppressions() {
    let root = case_root("advisories-respect-line-suppressions");
    let path = root.join("CLAUDE.md");
    let config = config_with_rule("{maxChars: 100, advisoryCharsRemaining: 100}");
    let files = vec![path];

    let advisories = advisories_with_files(&root, &config, &files).unwrap();

    assert!(advisories.is_empty());
}

#[test]
fn check_uses_custom_options() {
    let root = case_root("check-uses-custom-options");
    let config = config_with_rule("{maxLines: 5, maxChars: 1000}");
    let findings = check(&root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_uses_custom_filenames() {
    let root = case_root("check-uses-custom-filenames");
    // Without custom filename, GEMINI.md is not checked
    let config = config_with_rule("{maxLines: 2}");
    let findings = check(&root, &config).unwrap();
    assert!(findings.is_empty(), "GEMINI.md not in default set");
    // With custom filename
    let config2 = config_with_rule("{maxLines: 2, filenames: [\"GEMINI.md\"]}");
    let findings2 = check(&root, &config2).unwrap();
    assert_eq!(findings2.len(), 1);
}

#[test]
fn check_returns_empty_when_no_files() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_with_rule("{}");
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_file_skips_unreadable_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("AGENTS.md");
    // Path does not exist → read_to_string fails → returns empty
    let findings = check_file(&path, tmp.path(), 2, 100);
    assert!(findings.is_empty());
}

#[test]
fn check_with_files_respects_roots() {
    let root = case_root("check-with-files-respects-roots");
    let sub = root.join("sub");
    let outside = root.join("AGENTS.md");
    let inside = sub.join("AGENTS.md");
    let sub_str = sub.to_str().unwrap();
    let config = config_with_rule(&format!("{{maxLines: 2, roots: [\"{sub_str}\"]}}"));
    let all_files = vec![outside, inside];
    let findings = check_with_files(&root, &config, &all_files).unwrap();
    assert_eq!(
        findings.len(),
        1,
        "only the file within roots should be flagged"
    );
    assert!(findings[0].file.contains("sub"));
}

#[test]
fn check_with_files_normalizes_relative_roots() {
    let root = case_root("check-with-files-normalizes-relative-roots");
    let sub = root.join("sub");
    let outside = root.join("AGENTS.md");
    let inside = sub.join("AGENTS.md");
    let config = config_with_rule("{maxLines: 2, roots: [\"sub\"]}");
    let all_files = vec![outside, inside];
    let findings = check_with_files(&root, &config, &all_files).unwrap();
    assert_eq!(findings.len(), 1, "relative root resolves relative to root");
    assert!(findings[0].file.contains("sub"));
}

#[test]
fn check_sorts_findings_deterministically() {
    let root = case_root("check-sorts-findings-deterministically");
    let config = config_with_rule("{maxLines: 1, maxChars: 1}");
    let findings = check(&root, &config).unwrap();
    // findings should be sorted by file then message
    for i in 1..findings.len() {
        let a = (&findings[i - 1].file, &findings[i - 1].message);
        let b = (&findings[i].file, &findings[i].message);
        assert!(a <= b, "findings not sorted: {:?} > {:?}", a, b);
    }
}
