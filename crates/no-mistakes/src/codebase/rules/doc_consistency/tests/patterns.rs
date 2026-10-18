use super::super::{check, check_with_files, check_with_files_and_sources, RULE_ID};
use super::{config_with_rule, fixture_root};
use crate::codebase::rules::suppression::SuppressionDirectiveKind;
use crate::codebase::rules::{source_store_for_files, RuleFinding};
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::Path;

const PIN: &str = "Move this pin to versions.md.";
const SECRET: &str = "Move this secret block to versions.md.";
const REPEATED: &str = "repeated token";
const NONEMPTY: &str = "non-empty alternative is a finding";

fn patterns_config() -> (std::path::PathBuf, NoMistakesConfig) {
    let root = fixture_root("patterns");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    (root, config)
}

fn keys(findings: &[RuleFinding]) -> Vec<(String, usize, String)> {
    findings
        .iter()
        .map(|finding| (finding.file.clone(), finding.line, finding.message.clone()))
        .collect()
}

fn expected_raw() -> Vec<(&'static str, usize, &'static str)> {
    vec![
        ("docs/apostrophe-disabled.md", 2, PIN),
        ("docs/apostrophe-disabled.md", 3, PIN),
        ("docs/apostrophe-disabled.md", 4, PIN),
        ("docs/crlf.md", 3, PIN),
        ("docs/crlf.md", 5, PIN),
        ("docs/dash-disabled.md", 2, PIN),
        ("docs/dash-disabled.md", 3, PIN),
        ("docs/file-disabled.md", 2, PIN),
        ("docs/hash-disabled.md", 2, PIN),
        ("docs/hash-disabled.md", 3, PIN),
        ("docs/line-disabled.md", 2, PIN),
        ("docs/line-disabled.md", 3, PIN),
        (
            "docs/living.md",
            1,
            "docs/living.md: contains banned substring \"legacy API\"",
        ),
        ("docs/living.md", 3, PIN),
        ("docs/living.md", 5, PIN),
        ("docs/living.md", 7, SECRET),
        ("docs/living.md", 11, PIN),
        ("docs/next-disabled.md", 2, PIN),
        ("docs/next-disabled.md", 3, PIN),
        ("docs/overlap.md", 1, REPEATED),
        ("docs/overlap.md", 2, REPEATED),
        ("docs/slash-disabled.md", 2, PIN),
        ("docs/slash-disabled.md", 3, PIN),
        ("docs/zero-alt.md", 1, NONEMPTY),
    ]
}

fn config_with_rules(yamls: &[&str]) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    for yaml in yamls {
        config.rules.push(RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        });
    }
    config
}

fn assert_error_without_reads(root: &Path, yaml: &str, needle: &str) {
    let files = [root.join("CLAUDE.md")];
    let sources = source_store_for_files(&files);
    let error =
        check_with_files_and_sources(root, &config_with_rule(yaml), &files, &sources).unwrap_err();
    assert!(error.to_string().contains(needle), "{error}");
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn saved_patterns_report_real_match_lines() {
    let (root, config) = patterns_config();
    let findings = check(&root, &config).unwrap();
    let actual = keys(&findings);
    let expected: Vec<_> = expected_raw()
        .into_iter()
        .map(|(file, line, message)| (file.to_string(), line, message.to_string()))
        .collect();
    assert_eq!(actual, expected);
    let crlf_bytes = std::fs::read(root.join("docs/crlf.md")).unwrap();
    assert!(crlf_bytes.windows(2).any(|pair| pair == b"\r\n"));
}

#[test]
fn runner_suppresses_html_and_existing_comment_directives() {
    let (root, _) = patterns_config();
    let config_path = root.join(".no-mistakes.yml");
    let hidden = [
        (
            "docs/apostrophe-disabled.md",
            2,
            SuppressionDirectiveKind::Line,
            2,
        ),
        (
            "docs/apostrophe-disabled.md",
            3,
            SuppressionDirectiveKind::Line,
            3,
        ),
        (
            "docs/dash-disabled.md",
            2,
            SuppressionDirectiveKind::NextLine,
            1,
        ),
        (
            "docs/file-disabled.md",
            2,
            SuppressionDirectiveKind::File,
            1,
        ),
        (
            "docs/hash-disabled.md",
            2,
            SuppressionDirectiveKind::NextLine,
            1,
        ),
        (
            "docs/line-disabled.md",
            2,
            SuppressionDirectiveKind::Line,
            2,
        ),
        (
            "docs/next-disabled.md",
            2,
            SuppressionDirectiveKind::NextLine,
            1,
        ),
        (
            "docs/slash-disabled.md",
            2,
            SuppressionDirectiveKind::NextLine,
            1,
        ),
    ];
    let visible: Vec<_> = expected_raw()
        .into_iter()
        .filter(|finding| {
            !hidden
                .iter()
                .any(|hidden| hidden.0 == finding.0 && hidden.1 == finding.1)
        })
        .map(|(file, line, message)| (file.to_string(), line, message.to_string()))
        .collect();
    let plain = crate::check_runner::run_all_with_evidence(
        root.clone(),
        Some(config_path.clone()),
        None,
        false,
        false,
    )
    .unwrap();
    let plain_rules = keys(
        &plain
            .rules
            .iter()
            .filter(|finding| finding.rule == RULE_ID)
            .cloned()
            .collect::<Vec<_>>(),
    );
    assert_eq!(plain_rules, visible);
    assert!(plain.suppressed.is_empty());

    let audited =
        crate::check_runner::run_all_with_evidence(root, Some(config_path), None, true, false)
            .unwrap();
    let audited_rules = keys(
        &audited
            .rules
            .iter()
            .filter(|finding| finding.rule == RULE_ID)
            .cloned()
            .collect::<Vec<_>>(),
    );
    assert_eq!(audited_rules, visible);
    let mut recorded: Vec<_> = audited
        .suppressed
        .iter()
        .filter(|finding| finding.rule == RULE_ID)
        .map(|finding| {
            (
                finding.file.as_str(),
                finding.line,
                finding.directive.kind,
                finding.directive.line,
            )
        })
        .collect();
    recorded.sort();
    let mut expected_hidden: Vec<_> = hidden
        .iter()
        .map(|(file, line, kind, directive)| (*file, Some(*line), *kind, *directive))
        .collect();
    expected_hidden.sort();
    assert_eq!(recorded, expected_hidden);
}

#[test]
fn include_scope_intersects_pattern_globs() {
    let (root, mut config) = patterns_config();
    config.rules[0].include = vec!["docs/living.md".to_string()];
    let findings = check(&root, &config).unwrap();
    assert!(findings.iter().any(|finding| finding.file == "README.md"));
    assert!(findings
        .iter()
        .all(|finding| { finding.file == "docs/living.md" || finding.file == "README.md" }));
    assert!(findings
        .iter()
        .any(|finding| finding.file == "docs/living.md" && finding.line == 11));
    assert!(!findings
        .iter()
        .any(|finding| finding.file == "docs/crlf.md"));
}

#[test]
fn supplied_inventory_is_not_rediscovered() {
    let (root, config) = patterns_config();
    let crlf = root.join("docs/crlf.md");
    let findings = check_with_files(&root, &config, &[crlf]).unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding.file == "docs/crlf.md"));
    assert!(!findings
        .iter()
        .any(|finding| finding.file == "docs/living.md"));
    assert!(findings.iter().any(|finding| finding.file == "README.md"));
}

#[test]
fn substring_and_pattern_share_one_source_read() {
    let (root, config) = patterns_config();
    let living = vec![root.join("docs/living.md")];
    let sources = source_store_for_files(&living);
    let findings = check_with_files_and_sources(&root, &config, &living, &sources).unwrap();
    assert_eq!(sources.physical_read_count(), 1);
    assert!(findings.iter().any(|finding| finding.line == 1));
    assert!(findings.iter().any(|finding| finding.line == 7));

    let both = vec![root.join("README.md"), root.join("docs/living.md")];
    let sources = source_store_for_files(&both);
    let findings = check_with_files_and_sources(&root, &config, &both, &sources).unwrap();
    assert_eq!(sources.physical_read_count(), 2);
    assert!(!findings.iter().any(|finding| finding.file == "README.md"));
}

#[test]
fn empty_patterns_do_not_report_regex_matches() {
    let root = fixture_root("patterns");
    let findings = check(
        &root,
        &config_with_rule("bannedSubstrings: [\"legacy API\"]"),
    )
    .unwrap();
    assert_eq!(
        keys(&findings),
        vec![(
            "docs/living.md".to_string(),
            1,
            "docs/living.md: contains banned substring \"legacy API\"".to_string(),
        )]
    );
}

#[test]
fn invalid_patterns_fail_before_reads() {
    let root = fixture_root("pass");
    for (yaml, needle) in [
        (
            "bannedPatterns:\n  - pattern: PIN\n    message: hi\n    files: []",
            "bannedPatterns[0].files must list at least one glob",
        ),
        (
            "bannedPatterns:\n  - files: [\"docs/*.md\", \"\"]\n    pattern: PIN\n    message: hi",
            "bannedPatterns[0].files must not contain an empty glob",
        ),
        (
            "bannedPatterns:\n  - files: [\"   \"]\n    pattern: PIN\n    message: hi",
            "bannedPatterns[0].files must not contain an empty glob",
        ),
        (
            "bannedPatterns:\n  - files: [\"docs/*.md\"]\n    pattern: \"\"\n    message: hi",
            "bannedPatterns[0].pattern must not be empty",
        ),
        (
            "bannedPatterns:\n  - files: [\"docs/*.md\"]\n    pattern: PIN\n    message: \"\"",
            "bannedPatterns[0].message must not be empty",
        ),
        (
            "bannedPatterns:\n  - files: [\"docs/*.md\"]\n    pattern: PIN\n    message: \"   \"",
            "bannedPatterns[0].message must not be empty",
        ),
        (
            "bannedPatterns:\n  - files: [\"[\"]\n    pattern: PIN\n    message: hi",
            "bannedPatterns[0].files contains invalid glob",
        ),
        (
            "bannedPatterns:\n  - files: [\"docs/*.md\"]\n    pattern: PIN\n    message: hi\n  - files: [\"docs/*.md\"]\n    pattern: \"(\"\n    message: later",
            "bannedPatterns[1].pattern is an invalid regex",
        ),
    ] {
        assert_error_without_reads(&root, yaml, needle);
    }

    let sources = source_store_for_files(&[]);
    let error = check_with_files_and_sources(
        &root,
        &config_with_rule(
            "bannedPatterns:\n  - files: [\"docs/*.md\"]\n    pattern: \"(\"\n    message: hi",
        ),
        &[],
        &sources,
    )
    .unwrap_err();
    assert!(error.to_string().contains("bannedPatterns[0]"));
    assert!(error.to_string().contains("invalid regex"));
    assert_eq!(sources.physical_read_count(), 0);

    let invalid_root = fixture_root("patterns-invalid");
    let error = check(
        &invalid_root,
        &crate::config::v2::load_v2_config(
            &invalid_root,
            Some(&invalid_root.join(".no-mistakes.yml")),
        )
        .unwrap(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("bannedPatterns[0]"));
    assert!(error.to_string().contains("invalid regex"));
}

#[test]
fn valid_pattern_on_an_empty_selection_reads_nothing() {
    let root = fixture_root("pass");
    let sources = source_store_for_files(&[]);
    let findings = check_with_files_and_sources(
        &root,
        &config_with_rule(
            "bannedPatterns:\n  - files: [\"docs/**/*.md\"]\n    pattern: 'PIN-[0-9]{3}'\n    message: 'Move this pin to versions.md.'",
        ),
        &[],
        &sources,
    )
    .unwrap();
    assert!(findings.is_empty());
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn unreadable_path_is_skipped() {
    let root = fixture_root("patterns");
    let ghost = root.join("ghost.md");
    let findings = check_with_files(
        &root,
        &config_with_rule(
            "bannedSubstrings: [\"legacy API\"]\nbannedPatterns:\n  - files: [\"**/*.md\"]\n    pattern: PIN\n    message: pin",
        ),
        &[ghost],
    )
    .unwrap();
    assert!(findings.is_empty());
}

#[test]
fn missing_required_file_stays_when_patterns_are_set() {
    let root = fixture_root("pass");
    let findings = check(
        &root,
        &config_with_rule(
            "requiredFiles: [ABSENT.md]\nbannedPatterns:\n  - files: [\"**/*.md\"]\n    pattern: 'PIN-[0-9]{3}'\n    message: 'Move this pin to versions.md.'",
        ),
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "ABSENT.md");
    assert!(findings[0].message.contains("not found"));
}

#[test]
fn pattern_text_and_messages_keep_author_spacing() {
    let (root, _) = patterns_config();
    let living = [root.join("docs/living.md")];
    let padded = check_with_files(
        &root,
        &config_with_rule(
            "bannedPatterns:\n  - files: [\"  docs/living.md  \"]\n    pattern: \"\\tPIN-[0-9]{3}\"\n    message: \"  spaced message  \"",
        ),
        &living,
    )
    .unwrap();
    assert!(
        padded.is_empty(),
        "whitespace stays part of the regex: {padded:?}"
    );
    let trimmed = check_with_files(
        &root,
        &config_with_rule(
            "bannedPatterns:\n  - files: [\"  docs/living.md  \"]\n    pattern: 'PIN-[0-9]{3}'\n    message: \"  spaced message  \"",
        ),
        &living,
    )
    .unwrap();
    assert!(trimmed
        .iter()
        .all(|finding| finding.message == "spaced message"));
    assert!(trimmed.iter().any(|finding| finding.line == 3));
}

#[test]
fn separate_applications_compile_their_own_patterns() {
    let (root, _) = patterns_config();
    let overlap = [root.join("docs/overlap.md")];
    let entry = |pattern: &str, message: &str| {
        format!(
            "bannedPatterns:\n  - files: [\"docs/overlap.md\"]\n    pattern: {pattern}\n    message: {message}"
        )
    };
    let findings = check_with_files(
        &root,
        &config_with_rules(&[&entry("ZZ", "short token"), &entry("ZZZZ", "long token")]),
        &overlap,
    )
    .unwrap();
    assert_eq!(
        keys(&findings),
        vec![
            ("docs/overlap.md".to_string(), 1, "long token".to_string()),
            ("docs/overlap.md".to_string(), 1, "short token".to_string()),
            ("docs/overlap.md".to_string(), 2, "short token".to_string()),
        ]
    );
    let collapsed = check_with_files(
        &root,
        &config_with_rules(&[&entry("ZZ", "same token"), &entry("ZZ", "same token")]),
        &overlap,
    )
    .unwrap();
    assert_eq!(collapsed.len(), 2);

    let invalid = config_with_rules(&[
        &entry("ZZ", "short token"),
        "bannedPatterns:\n  - files: [\"docs/overlap.md\"]\n    pattern: \"(\"\n    message: broken",
    ]);
    assert!(check_with_files(&root, &invalid, &overlap).is_err());
}
