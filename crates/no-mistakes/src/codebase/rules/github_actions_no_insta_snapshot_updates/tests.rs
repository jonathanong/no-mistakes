use super::{check_with_files, diagnostics_with_files_and_sources, RULE_ID};
use crate::codebase::rules::source_store_for_files;
use crate::config::v2::{load_v2_config, Unanalyzable};
use std::path::PathBuf;

fn config(name: &str) -> crate::config::v2::NoMistakesConfig {
    let fixture = crate::test_support::rule_fixture_root(RULE_ID);
    let config_path = fixture.parent().unwrap().join("configs").join(name);
    load_v2_config(&fixture, Some(&config_path)).unwrap()
}

#[test]
fn check_with_files_applies_inline_and_file_suppressions() {
    let fixture = crate::test_support::rule_fixture_root(RULE_ID);
    let root = fixture.join("suppression");
    let config = config("enabled.yml");
    let files: Vec<PathBuf> = [
        root.join(".github/workflows/check.yml"),
        root.join(".github/workflows/regenerate.yml"),
    ]
    .into_iter()
    .collect();
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".github/workflows/check.yml");
    assert_eq!(findings[0].line, 16);
}

#[test]
fn missing_workflow_reports_read_failure_through_unanalyzable_policy() {
    let root = crate::test_support::rule_fixture_root(RULE_ID).join("demo");
    let missing = [root.join(".github/workflows/missing.yml")];
    let sources = source_store_for_files(&missing);
    for (policy, failures, advisories) in [
        (Unanalyzable::Report, 1, 0),
        (Unanalyzable::Advise, 0, 1),
        (Unanalyzable::Ignore, 0, 0),
    ] {
        let mut cfg = config("enabled.yml");
        cfg.unanalyzable = Some(policy);
        let out =
            diagnostics_with_files_and_sources(&root, &cfg, &missing, &sources, false).unwrap();
        assert_eq!((out.findings.len(), out.advisories.len()), (failures, advisories));
        for diagnostic in out.findings.iter().chain(&out.advisories) {
            assert_eq!(diagnostic.line, 1);
            assert_eq!(diagnostic.target.as_deref(), Some("unanalyzable"));
            assert!(diagnostic.message.contains("cannot read source"));
        }
    }
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn invalid_utf8_workflow_reports_byte_line_and_honors_file_suppression() {
    let root = crate::test_support::rule_fixture_root(RULE_ID).join("unreadable");
    let files = [
        root.join(".github/workflows/invalid-utf8.yml"),
        root.join(".github/workflows/suppressed.yml"),
    ];
    let sources = source_store_for_files(&files);
    let out = diagnostics_with_files_and_sources(
        &root,
        &config("enabled.yml"),
        &files,
        &sources,
        false,
    )
    .unwrap();
    assert_eq!(out.findings.len(), 1, "{:?}", out.findings);
    assert!(out.advisories.is_empty());
    assert_eq!(out.findings[0].file, ".github/workflows/invalid-utf8.yml");
    assert_eq!(out.findings[0].line, 2);
    assert_eq!(out.findings[0].target.as_deref(), Some("unanalyzable"));
    assert_eq!(sources.physical_read_count(), 2);
}

#[test]
fn parse_error_uses_real_line_and_custom_message_overrides_output() {
    let root = crate::test_support::rule_fixture_root(RULE_ID).join("unanalyzable");
    let files = [root.join(".github/workflows/syntax-line.yml")];
    let sources = source_store_for_files(&files);
    let out = diagnostics_with_files_and_sources(
        &root,
        &config("custom-message.yml"),
        &files,
        &sources,
        false,
    )
    .unwrap();
    assert_eq!(out.findings.len(), 1, "{:?}", out.findings);
    assert_eq!(out.findings[0].line, 5);
    assert_eq!(
        out.findings[0].message,
        "Custom snapshot update policy message."
    );
}

#[test]
fn yaml_merge_keys_report_unanalyzable_in_workflow_jobs_and_env() {
    let root = crate::test_support::rule_fixture_root(RULE_ID).join("merge-keys");
    let files = [root.join(".github/workflows/merge.yml")];
    let sources = source_store_for_files(&files);
    let out = diagnostics_with_files_and_sources(
        &root,
        &config("enabled.yml"),
        &files,
        &sources,
        false,
    )
    .unwrap();
    let lines: Vec<_> = out.findings.iter().map(|f| f.line).collect();
    assert_eq!(lines, [3, 5, 9], "{:?}", out.findings);
    assert!(out
        .findings
        .iter()
        .all(|f| f.target.as_deref() == Some("unanalyzable")));
}
