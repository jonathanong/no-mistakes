use super::*;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root(RULE_ID).join("patterns"),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(crate::config::v2::schema::RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

#[test]
fn saved_patterns_keep_scopes_lines_and_custom_messages() {
    let root = fixture();
    let cfg =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let findings = check(&root, &cfg).unwrap();
    let locations: Vec<_> = findings.iter().map(|f| (f.file.as_str(), f.line)).collect();
    assert_eq!(
        locations,
        [
            ("README.md", 4),
            ("docs/generated.md", 2),
            ("docs/guide.md", 3),
            ("docs/guide.md", 4),
            ("docs/guide.md", 6),
            ("docs/suppressed.md", 2),
            ("docs/suppressed.md", 3),
            ("docs/suppressed.md", 4),
        ]
    );
    assert_eq!(
        findings[0].message,
        "Keep live pins in tools/pins.json; link that owner file."
    );
}

#[test]
fn prepared_source_is_read_once_for_literal_and_regex_policies() {
    let root = fixture();
    let file = root.join("README.md");
    let files = [file];
    let sources = super::super::source_store_for_files(&files);
    let cfg = config("bannedSubstrings: ['Current SHA']\nbannedPatterns:\n  - files: [README.md]\n    pattern: 'Current SHA'\n    message: owner");
    let first = check_with_files_and_sources(&root, &cfg, &files, &sources).unwrap();
    assert_eq!(first.len(), 2);
    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(
        first
            .iter()
            .find(|f| f.message.contains("banned substring"))
            .unwrap()
            .line,
        1
    );
    assert_eq!(first.iter().find(|f| f.message == "owner").unwrap().line, 4);
    assert_eq!(
        check_with_files_and_sources(&root, &cfg, &files, &sources).unwrap(),
        first
    );
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn empty_or_unmatched_pattern_scope_never_reads_sources() {
    let root = fixture();
    let files = [root.join("README.md")];
    let sources = super::super::source_store_for_files(&files);
    for yaml in [
        "{}",
        "bannedPatterns: []",
        "bannedPatterns: [{files: ['docs/*.md'], pattern: pin, message: owner}]",
    ] {
        assert!(
            check_with_files_and_sources(&root, &config(yaml), &files, &sources)
                .unwrap()
                .is_empty()
        );
    }
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn invalid_pattern_configuration_errors_without_selected_files() {
    for yaml in [
        "bannedPatterns: [{files: [README.md], pattern: '[', message: owner}]",
        "bannedPatterns: [{files: ['['], pattern: pin, message: owner}]",
        "bannedPatterns: [{files: [], pattern: pin, message: owner}]",
        "bannedPatterns: [{files: [' '], pattern: pin, message: owner}]",
        "bannedPatterns: [{files: [README.md], pattern: '', message: owner}]",
        "bannedPatterns: [{files: [README.md], pattern: pin, message: ' '}]",
        "bannedPatterns: [{files: [README.md], pattern: pin}]",
        "bannedPatterns: [{files: [README.md], pattern: pin, message: owner, typo: true}]",
    ] {
        assert!(
            check_with_files(&fixture(), &config(yaml), &[]).is_err(),
            "{yaml}"
        );
    }
}

#[test]
fn pattern_line_suppression_uses_normal_filesystem_pipeline() {
    let root = fixture();
    // Standalone filesystem rules return raw findings; the shared check boundary
    // owns suppression and its request-scoped source inventory.
    let findings = crate::check_runner::run_all_with_evidence(
        root.clone(),
        Some(root.join(".no-mistakes.yml")),
        None,
        false,
        false,
    )
    .unwrap()
    .rules;
    assert_eq!(findings.len(), 5, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.file == "docs/suppressed.md")
            .map(|f| f.line)
            .collect::<Vec<_>>(),
        [4]
    );
    assert!(!findings.iter().any(|f| f.file == "docs/generated.md"));
}

#[test]
fn regex_flags_unicode_crlf_and_eof_keep_start_lines() {
    let root = fixture();
    let path = root.join("docs/line-shapes.md");
    let files = [path];
    for (pattern, expected) in [
        (r"\bpin\b", vec![2, 6]),
        (r"(?im)^pin", vec![2, 5, 6]),
        (r"Start\r?\nEnd", vec![3]),
        (r"(?m)^", vec![1, 2, 3, 4, 5, 6]),
    ] {
        let yaml = format!(
            "bannedPatterns: [{{files: ['docs/*.md'], pattern: '{pattern}', message: owner}}]"
        );
        let actual = check_with_files(&root, &config(&yaml), &files).unwrap();
        assert_eq!(
            actual.iter().map(|f| f.line).collect::<Vec<_>>(),
            expected,
            "{pattern}"
        );
    }
}

#[test]
fn configured_unreadable_source_keeps_shared_failure_identity() {
    let root = fixture();
    let files = [root.join("not-on-disk.md")];
    let sources = super::super::source_store_for_files(&files);
    let cfg = config("bannedPatterns: [{files: ['*.md'], pattern: pin, message: owner}]");
    for _ in 0..2 {
        assert!(check_with_files_and_sources(&root, &cfg, &files, &sources)
            .unwrap()
            .is_empty());
    }
    assert_eq!(sources.physical_read_count(), 1);
}
