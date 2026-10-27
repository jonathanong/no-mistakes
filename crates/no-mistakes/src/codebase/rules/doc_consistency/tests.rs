use super::*;
use crate::codebase::rules::source_store_for_files;
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

fn fixture_root(subpath: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/doc-consistency/fixture")
            .join(subpath),
    )
}

fn case_root(name: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("doc-consistency")
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
        "expected findings for missing heading"
    );
}

#[test]
fn missing_required_file_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let config = config_with_rule("requiredFiles: [NONEXISTENT.md]");
    let findings = check(root, &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("not found"));
}

#[test]
fn required_heading_present_passes() {
    let root = case_root("required-heading-present-passes");
    let config =
        config_with_rule("requiredFiles: [GUIDE.md]\nrequiredHeading: \"## Related issues\"");
    let findings = check(&root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn required_heading_missing_flagged() {
    let root = case_root("required-heading-missing-flagged");
    let config =
        config_with_rule("requiredFiles: [GUIDE.md]\nrequiredHeading: \"## Related issues\"");
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("missing required heading"));
}

#[test]
fn required_substring_present_passes() {
    let root = case_root("required-substring-present-passes");
    let config = config_with_rule(
        "requiredFiles: [README.md]\nrequiredSubstrings:\n  - file: README.md\n    substring: CONTRIBUTING.md",
    );
    let findings = check(&root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn required_substring_missing_flagged() {
    let root = case_root("required-substring-missing-flagged");
    let config = config_with_rule(
        "requiredFiles: [README.md]\nrequiredSubstrings:\n  - file: README.md\n    substring: CONTRIBUTING.md",
    );
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("missing required substring"));
}

#[test]
fn banned_substring_detected() {
    let root = case_root("banned-substring-detected");
    let config = config_with_rule("bannedSubstrings: [\"legacy API\"]");
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("banned substring"));
}

#[test]
fn banned_substring_absent_passes() {
    let root = case_root("banned-substring-absent-passes");
    let config = config_with_rule("bannedSubstrings: [\"legacy API\"]");
    let findings = check(&root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_with_files_works() {
    let root = case_root("check-with-files-works");
    let guide = root.join("GUIDE.md");
    let config =
        config_with_rule("requiredFiles: [GUIDE.md]\nrequiredHeading: \"## Related issues\"");
    let findings = check_with_files(&root, &config, &[guide]).unwrap();
    assert_eq!(findings.len(), 1);
}

#[test]
fn empty_options_has_no_findings() {
    let root = case_root("empty-options-has-no-findings");
    let config = config_with_rule("{}");
    let findings = check(&root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn required_file_exists_but_unreadable_skips_content_checks() {
    // File is in the rel_set (passed as a tracked file) but cannot be read
    // (non-existent on disk).  The scan should skip content checks and not panic.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // Register a file path that doesn't actually exist on disk
    let ghost = root.join("GHOST.md");
    let config = config_with_rule("requiredFiles: [GHOST.md]\nrequiredHeading: \"## Title\"");
    // Pass the ghost path so it appears in the rel_set — it "exists" per tracking
    let findings = check_with_files(root, &config, &[ghost]).unwrap();
    // Content checks are skipped with `continue` when read_to_string fails,
    // so no heading-missing finding should be emitted.
    assert!(
        findings.is_empty(),
        "unreadable required file should produce no content findings, got: {findings:?}"
    );
}

#[test]
fn required_substring_spec_for_different_file_is_skipped() {
    // The substring spec targets "OTHER.md" but we're checking "README.md".
    // The inner `if spec.file != *req_file { continue; }` branch must fire.
    let root = case_root("required-substring-spec-for-different-file-is-skipped");
    let config = config_with_rule(
        "requiredFiles: [README.md]\nrequiredSubstrings:\n  - file: OTHER.md\n    substring: something",
    );
    let findings = check_with_files(&root, &config, &[root.join("README.md")]).unwrap();
    assert!(
        findings.is_empty(),
        "substring spec for a different file should be skipped"
    );
}

#[test]
fn banned_substring_in_unreadable_file_skipped() {
    // Files listed in `files` that cannot be read should be silently skipped
    // in the banned_substrings loop.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let ghost = root.join("ghost.md");
    // Do NOT create ghost.md on disk — read will fail
    let config = config_with_rule("bannedSubstrings: [\"badword\"]");
    let findings = check_with_files(root, &config, &[ghost]).unwrap();
    assert!(
        findings.is_empty(),
        "unreadable file in banned_substrings loop should be silently skipped"
    );
}

#[test]
fn invalid_options_and_empty_rule_set() {
    let root = fixture_root("pass");
    let mut invalid = NoMistakesConfig::default();
    invalid.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("true").unwrap(),
        ..Default::default()
    });
    assert!(check(&root, &invalid).is_err());
    assert!(check(&root, &NoMistakesConfig::default())
        .unwrap()
        .is_empty());

    let files = [root.join("CLAUDE.md")];
    let sources = super::super::source_store_for_files(&files);
    let findings =
        check_with_files_and_sources(&root, &config_with_rule("{}"), &files, sources.as_ref())
            .unwrap();
    assert!(findings.is_empty());
}

#[test]
fn entrypoint_option_and_include_errors_propagate() {
    let root = fixture_root("pass");
    let files = [root.join("CLAUDE.md")];
    let mut invalid_include = config_with_rule("{}");
    invalid_include.rules[0].include = vec!["[".to_string()];
    assert!(check(&root, &invalid_include).is_err());
    assert!(check_with_files(&root, &invalid_include, &files).is_err());

    let mut invalid_options = NoMistakesConfig::default();
    invalid_options.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("true").unwrap(),
        ..Default::default()
    });
    assert!(check_with_files(&root, &invalid_options, &files).is_err());
}

fn load_pattern_fixture(name: &str) -> (std::path::PathBuf, NoMistakesConfig) {
    let root = fixture_root(&format!("pattern-config/{name}"));
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .unwrap_or_else(|error| panic!("{name} config should load: {error}"));
    (root, config)
}

fn pattern_config_error(name: &str) -> String {
    let (root, config) = load_pattern_fixture(name);
    let files = [root.join("CLAUDE.md")];
    let sources = source_store_for_files(&files);
    let error = check_with_files_and_sources(&root, &config, &files, &sources).unwrap_err();
    assert_eq!(sources.physical_read_count(), 0, "{error}");
    error.to_string()
}

#[test]
fn empty_matching_patterns_fail_at_config_load() {
    let root = fixture_root("patterns-empty");
    let error = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap(),
    )
    .unwrap_err();
    let message = error.to_string();
    assert!(message.contains("bannedPatterns[0]"), "{message}");
    assert!(message.contains("`a*|QQ`"), "{message}");
    assert!(message.contains("hides later alternatives"), "{message}");

    for (name, pattern) in [
        ("a-star", "a*"),
        ("a-optional", "a?"),
        ("optional-group", "(a|)?"),
        ("empty-alt", "(|b)"),
        ("multiline-start", "(?m)^"),
        ("star-or-qq", "a*|QQ"),
        ("qq-or-star", "QQ|a*"),
        ("nested-empty-alt-under-star", "(?:(|b)c)*"),
        // Concat of empty pieces. `()(|b)` reaches the hidden alternative
        // through the all-empty concat; `()()` is empty with no alternation.
        ("empty-concat-alt", "()(|b)"),
        ("empty-concat", "()()"),
    ] {
        let message = pattern_config_error(name);
        assert!(
            message.contains(&format!("`{pattern}`")),
            "{name}: {message}"
        );
        let hides_alternative = matches!(
            name,
            "optional-group" | "empty-alt" | "star-or-qq" | "qq-or-star" | "empty-concat-alt"
        );
        assert_eq!(
            message.contains("hides later alternatives"),
            hides_alternative,
            "{name}: {message}"
        );
        if !hides_alternative {
            assert!(
                message.contains("empty matches are never reported"),
                "{name}: {message}"
            );
        }
    }
    let later = pattern_config_error("later-index");
    assert!(later.contains("bannedPatterns[1]"), "{later}");
    assert!(later.contains("`a*|QQ`"), "{later}");
    assert!(later.contains("hides later alternatives"), "{later}");
}

#[test]
fn non_empty_patterns_still_compile() {
    for name in [
        "compile-a-plus",
        "compile-optional-star-pin",
        "compile-intersection",
    ] {
        let (root, config) = load_pattern_fixture(name);
        check(&root, &config).unwrap_or_else(|error| panic!("{name} should compile: {error}"));
    }
}

#[test]
fn oversized_patterns_report_compiled_size_limit() {
    let message = pattern_config_error("too-big");
    assert!(message.contains("`(a{1000}){1000}`"), "{message}");
    assert!(message.contains("10 MiB compiled-size limit"), "{message}");
    assert!(message.contains("reduce repetition counts"), "{message}");
    assert!(
        !message.contains("invalid regex"),
        "size failures must not look like syntax errors: {message}"
    );
}

#[test]
fn empty_engine_matches_are_not_findings() {
    let regex = regex::Regex::new("a*").unwrap();
    assert!(super::patterns::match_start_lines(&regex, "QQ").is_empty());
    assert_eq!(super::patterns::match_start_lines(&regex, "ba"), vec![1]);
}

mod patterns;

#[test]
fn required_heading_uses_parsed_level_and_ignores_code_examples() {
    let root = fixture_root("heading-levels");
    let files = [
        "exact.md",
        "shallow.md",
        "deep.md",
        "fenced.md",
        "indented.md",
    ]
    .map(|name| root.join(name));
    let config = config_with_rule(
        "requiredFiles: [exact.md, shallow.md, deep.md, fenced.md, indented.md]\nrequiredHeading: \"## Related issues\"",
    );
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["deep.md", "fenced.md", "indented.md", "shallow.md"],
        "only a parsed heading at the configured level satisfies the requirement"
    );

    for heading in [
        "Related issues",
        "#Related issues",
        "####### Related issues",
    ] {
        // Legacy non-ATX hash prefixes retain bare-name matching at any level.
        let config = config_with_rule(&format!(
            "requiredFiles: [exact.md, shallow.md, deep.md]\nrequiredHeading: {heading:?}"
        ));
        assert!(
            check_with_files(&root, &config, &files).unwrap().is_empty(),
            "{heading}"
        );
    }
}
