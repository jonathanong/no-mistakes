use super::prepared::{
    count_physical_lines, is_test_file, DEFAULT_SRC_MAX, DEFAULT_TEST_MAX, DEFAULT_TEST_ROOTS,
};
use super::scan::{check_file, check_source};
use super::*;
use crate::codebase::rules::path_filter::GlobMatcher;
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

fn fixture(path: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/csharp-max-lines-per-file/fixture")
            .join(path),
    )
}

fn default_test_roots() -> GlobMatcher {
    GlobMatcher::new(
        &DEFAULT_TEST_ROOTS
            .iter()
            .map(|pattern| (*pattern).to_string())
            .collect::<Vec<_>>(),
        "testRoots",
    )
    .unwrap()
}

#[test]
fn count_physical_lines_empty() {
    assert_eq!(count_physical_lines(""), 0);
}

#[test]
fn count_physical_lines_single_no_newline() {
    assert_eq!(count_physical_lines("hello"), 1);
}

#[test]
fn count_physical_lines_trailing_newline() {
    assert_eq!(count_physical_lines("a\nb\n"), 2);
}

#[test]
fn count_physical_lines_no_trailing_newline() {
    assert_eq!(count_physical_lines("a\nb"), 2);
}

#[test]
fn count_physical_lines_includes_comments_and_blanks() {
    assert_eq!(count_physical_lines("// a\n\n// b\n"), 3);
}

#[test]
fn is_test_file_tests_dir() {
    let matcher = default_test_roots();
    assert!(is_test_file(
        Path::new("/project"),
        Path::new("/project/src/tests/Helper.cs"),
        &matcher
    ));
    assert!(is_test_file(
        Path::new("/project"),
        Path::new("/project/tests/Helper.cs"),
        &matcher
    ));
}

#[test]
fn is_test_file_tests_project_folder() {
    let matcher = default_test_roots();
    assert!(is_test_file(
        Path::new("/project"),
        Path::new("/project/MyApp.Tests/ServiceTests.cs"),
        &matcher
    ));
    assert!(is_test_file(
        Path::new("/project"),
        Path::new("/project/src/MyApp.Tests/ServiceTests.cs"),
        &matcher
    ));
}

#[test]
fn is_test_file_does_not_use_rust_tests_rs_heuristic() {
    let matcher = default_test_roots();
    assert!(!is_test_file(
        Path::new("/project"),
        Path::new("/project/src/module/tests.rs"),
        &matcher
    ));
    assert!(!is_test_file(
        Path::new("/project"),
        Path::new("/project/src/module/tests.cs"),
        &matcher
    ));
    assert!(!is_test_file(
        Path::new("/project"),
        Path::new("/project/tests.cs"),
        &matcher
    ));
}

#[test]
fn is_test_file_src_file() {
    let matcher = default_test_roots();
    assert!(!is_test_file(
        Path::new("/project"),
        Path::new("/project/src/App.cs"),
        &matcher
    ));
    assert!(!is_test_file(
        Path::new("/project"),
        Path::new("/project/src/FooTests.cs"),
        &matcher
    ));
}

#[test]
fn is_test_file_custom_test_roots() {
    let matcher = GlobMatcher::new(&["**/Spec/**".to_string()], "testRoots").unwrap();
    assert!(is_test_file(
        Path::new("/project"),
        Path::new("/project/Spec/Foo.cs"),
        &matcher
    ));
    assert!(!is_test_file(
        Path::new("/project"),
        Path::new("/project/MyApp.Tests/Foo.cs"),
        &matcher
    ));
    assert!(is_test_file(
        Path::new("/project"),
        Path::new("/project/src/tests/Foo.cs"),
        &matcher
    ));
}

#[test]
fn check_source_uses_physical_lines_and_default_src_max() {
    let content = "a\n".repeat(DEFAULT_SRC_MAX + 1);
    let finding = check_source(
        Path::new("Foo.cs"),
        Path::new("/"),
        &content,
        DEFAULT_SRC_MAX,
        false,
    )
    .unwrap();
    assert!(finding.message.contains(&format!(
        "{} physical lines (max {DEFAULT_SRC_MAX})",
        DEFAULT_SRC_MAX + 1
    )));
    assert!(!finding.message.contains("code lines"));
}

#[test]
fn check_source_passes_within_limit() {
    assert!(check_source(Path::new("Foo.cs"), Path::new("/"), "a\nb\n", 5, false).is_none());
}

#[test]
fn check_source_empty_file_is_under_limit() {
    assert!(check_source(Path::new("Foo.cs"), Path::new("/"), "", 0, false).is_none());
}

#[test]
fn check_passes_within_src_limit() {
    let root = fixture("pass");
    let findings = check(&root, &config_with_rule("{srcMax: 20}")).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_fails_over_src_limit() {
    let root = fixture("fail");
    let findings = check(&root, &config_with_rule("{srcMax: 3}")).unwrap();
    assert!(findings.iter().any(|finding| finding.file == "TooLong.cs"));
    assert!(findings
        .iter()
        .any(|finding| finding.file == "CommentsOnly.cs"));
    assert!(findings
        .iter()
        .all(|finding| finding.message.contains("physical lines")));
    assert!(findings
        .iter()
        .all(|finding| !finding.message.contains("code lines")));
}

#[test]
fn check_sorts_multiple_findings_by_file() {
    let root = fixture("fail");
    let findings = check(&root, &config_with_rule("{srcMax: 3}")).unwrap();
    let files: Vec<&str> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    assert!(files.contains(&"CommentsOnly.cs"));
    assert!(files.contains(&"TooLong.cs"));
    assert!(files.contains(&"sub/TooLong.cs"));
    assert!(!files.contains(&"Suppressed.cs"));
    let mut sorted = files.clone();
    sorted.sort_unstable();
    assert_eq!(files, sorted);
}

#[test]
fn check_uses_test_limit_for_test_roots() {
    let root = fixture("test");
    let findings = check(&root, &config_with_rule("{srcMax: 3, testMax: 20}")).unwrap();
    assert!(findings.is_empty(), "test files should use testMax");
}

#[test]
fn check_fails_test_files_over_test_max() {
    let root = fixture("test");
    let findings = check(&root, &config_with_rule("{srcMax: 3, testMax: 3}")).unwrap();
    assert_eq!(findings.len(), 2);
    assert!(findings
        .iter()
        .any(|finding| finding.file == "MyApp.Tests/ServiceTests.cs"));
    assert!(findings
        .iter()
        .any(|finding| finding.file == "tests/LongTest.cs"));
}

#[test]
fn check_skips_generated_g_cs_by_default() {
    let root = fixture("generated");
    let findings = check(&root, &config_with_rule("{srcMax: 3}")).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_respects_excludes() {
    let root = fixture("fail");
    let findings = check(
        &root,
        &config_with_rule("{srcMax: 3, excludes: [\"TooLong\", \"CommentsOnly\"]}"),
    )
    .unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_file_skips_unreadable_file() {
    let root = fixture("pass");
    let path = root.join("missing.cs");
    assert!(check_file(&path, &root, 5, false).is_none());
}

#[test]
fn check_with_files_respects_roots() {
    let root = fixture("fail");
    let outside = root.join("TooLong.cs");
    let inside = root.join("sub/TooLong.cs");
    let config = config_with_rule("{srcMax: 3, roots: [\"sub\"]}");
    let findings = check_with_files(&root, &config, &[outside, inside]).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "sub/TooLong.cs");
}

#[test]
fn check_with_files_normalizes_relative_roots() {
    let root = fixture("fail");
    let config = config_with_rule("{srcMax: 3, roots: [\"sub\"]}");
    let findings = check_with_files(
        &root,
        &config,
        &[root.join("TooLong.cs"), root.join("sub/TooLong.cs")],
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].file.contains("sub"));
}

#[test]
fn check_with_files_normalizes_absolute_roots() {
    let root = fixture("fail");
    let sub = root.join("sub");
    let config = config_with_rule(&format!(
        "{{srcMax: 3, roots: [\"{}\"]}}",
        sub.to_str().unwrap()
    ));
    let findings = check_with_files(
        &root,
        &config,
        &[root.join("TooLong.cs"), root.join("sub/TooLong.cs")],
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "sub/TooLong.cs");
}

#[test]
fn check_with_files_ignores_non_csharp() {
    let root = fixture("fail");
    let findings = check_with_files(
        &root,
        &config_with_rule("{srcMax: 3}"),
        &[root.join("README.md"), root.join(".no-mistakes.yml")],
    )
    .unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_with_files_empty_roots_scans_nothing() {
    let root = fixture("fail");
    let findings = check_with_files(
        &root,
        &config_with_rule("{srcMax: 3, roots: []}"),
        &[root.join("TooLong.cs")],
    )
    .unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_with_files_and_sources_reads_once() {
    let root = fixture("fail");
    let files = vec![root.join("TooLong.cs")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let findings =
        check_with_files_and_sources(&root, &config_with_rule("{srcMax: 3}"), &files, &sources)
            .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn check_with_files_skips_missing_source() {
    let root = fixture("fail");
    let files = vec![root.join("missing.cs")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let findings =
        check_with_files_and_sources(&root, &config_with_rule("{srcMax: 3}"), &files, &sources)
            .unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_respects_disable_file_comment() {
    let root = fixture("fail");
    let findings = check(&root, &config_with_rule("{srcMax: 3}")).unwrap();
    assert!(findings
        .iter()
        .all(|finding| finding.file != "Suppressed.cs"));
}

#[test]
fn deferred_suppression_still_emits_disabled_file() {
    let root = fixture("fail");
    let files = vec![root.join("Suppressed.cs")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let findings = check_with_files_sources_and_deferred_suppression(
        &root,
        &config_with_rule("{srcMax: 3}"),
        &files,
        &sources,
        true,
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "Suppressed.cs");
}

#[test]
fn check_rejects_invalid_exclude_glob() {
    let root = fixture("pass");
    let error = check(&root, &config_with_rule("{excludes: [\"[\"]}"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("csharp-max-lines-per-file excludes"));
}

#[test]
fn check_rejects_invalid_test_root_glob() {
    let root = fixture("pass");
    let error = check(&root, &config_with_rule("{testRoots: [\"[\"]}"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("csharp-max-lines-per-file testRoots"));
}

#[test]
fn check_unconfigured_returns_empty() {
    let root = fixture("fail");
    let findings = check(&root, &NoMistakesConfig::default()).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn default_limits_match_rust_max_lines() {
    assert_eq!(DEFAULT_SRC_MAX, 200);
    assert_eq!(DEFAULT_TEST_MAX, 500);
}

#[test]
fn empty_test_roots_still_honor_tests_path() {
    let matcher = GlobMatcher::new(&[] as &[String], "testRoots").unwrap();
    assert!(is_test_file(
        Path::new("/project"),
        Path::new("/project/src/tests/Foo.cs"),
        &matcher
    ));
    assert!(!is_test_file(
        Path::new("/project"),
        Path::new("/project/MyApp.Tests/Foo.cs"),
        &matcher
    ));
}

#[test]
fn check_with_files_uses_options_per_rule_application() {
    let root = fixture("fail");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("{srcMax: 100, roots: [\"sub\"]}").unwrap(),
        ..Default::default()
    });
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("{srcMax: 3, roots: [\"sub\"]}").unwrap(),
        ..Default::default()
    });
    let findings = check_with_files(
        &root,
        &config,
        &[root.join("TooLong.cs"), root.join("sub/TooLong.cs")],
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "sub/TooLong.cs");
    assert!(findings[0].message.contains("max 3"));
}

#[test]
fn custom_test_roots_replace_defaults() {
    let root = fixture("test");
    let findings = check(
        &root,
        &config_with_rule("{srcMax: 3, testMax: 20, testRoots: [\"**/Spec/**\"]}"),
    )
    .unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding.file == "MyApp.Tests/ServiceTests.cs"));
    assert!(findings
        .iter()
        .all(|finding| finding.file != "tests/LongTest.cs"));
}
