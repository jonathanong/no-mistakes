use super::*;

/// Cover the false branches of the `if rule_enabled(...)` guards for
/// `RUST_MAX_LINES_PER_FILE` and `RUST_NO_INLINE_TESTS` by running with a
/// config that omits those two rules, exercising the skip paths.
#[test]
fn dispatch_with_files_skips_disabled_rules() {
    let fixture = all_rules_fixture();
    // Omit RUST_MAX_LINES_PER_FILE and RUST_NO_INLINE_TESTS from the config.
    let rules_without_rust: Vec<&str> = FILESYSTEM_RULE_IDS
        .iter()
        .copied()
        .filter(|&rule| rule != RUST_MAX_LINES_PER_FILE && rule != RUST_NO_INLINE_TESTS)
        .collect();
    let config_path = write_config(fixture.path(), &rules_without_rust);
    let findings =
        run_filesystem_rules_with_files(fixture.path(), Some(&config_path), &[]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn standalone_entrypoint_returns_configuration_errors() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/filesystem-dispatch/invalid-config");
    let error = run_filesystem_rules(&root, Some(&root.join(".no-mistakes.yml"))).unwrap_err();
    assert!(error.to_string().contains("parse"), "{error:#}");
    let supplied =
        run_filesystem_rules_with_files(&root, Some(&root.join(".no-mistakes.yml")), &[])
            .unwrap_err();
    assert!(supplied.to_string().contains("parse"), "{supplied:#}");
}

#[test]
fn dispatch_uses_fallback_for_an_unknown_rule() {
    fn fallback(
        _root: &std::path::Path,
        _config: &crate::config::v2::NoMistakesConfig,
        _files: &[std::path::PathBuf],
    ) -> anyhow::Result<Vec<RuleFinding>> {
        Ok(vec![RuleFinding {
            rule: "fallback".to_string(),
            file: "fixture.txt".to_string(),
            line: 1,
            message: "fallback rule ran".to_string(),
            import: None,
            target: None,
        }])
    }

    let root = std::path::Path::new("/fixture");
    let config = crate::config::v2::NoMistakesConfig::default();
    let files = Vec::new();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(root, &files);
    let findings = super::run_rule::run_rule_with_sources(super::run_rule::RunRuleRequest {
        rule_id: "future-filesystem-rule",
        fallback,
        root,
        config: &config,
        files: &files,
        snapshot: &snapshot,
        sources: &sources,
        facts: None,
        defer_suppression: false,
    })
    .unwrap();

    assert_eq!(findings[0].rule, "fallback");
}

#[test]
fn run_rule_dispatches_markdown_link_display_text() {
    fn fallback(
        _root: &std::path::Path,
        _config: &crate::config::v2::NoMistakesConfig,
        _files: &[std::path::PathBuf],
    ) -> anyhow::Result<Vec<RuleFinding>> {
        panic!("markdown-link-display-text should not use the fallback");
    }

    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check-runner/empty");
    let config = crate::config::v2::NoMistakesConfig::default();
    let files = Vec::new();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &files);
    let findings = super::run_rule::run_rule_with_sources(super::run_rule::RunRuleRequest {
        rule_id: MARKDOWN_LINK_DISPLAY_TEXT,
        fallback,
        root: &root,
        config: &config,
        files: &files,
        snapshot: &snapshot,
        sources: &sources,
        facts: None,
        defer_suppression: false,
    })
    .unwrap();

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn mermaid_invalid_include_glob_fails_markdown_prepare() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/markdown-mermaid-validation");
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    config.rules[0].include = vec!["[".to_string()];
    let error = run_filesystem_rules_with_config(&root, &config, &[]).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("rule `markdown-mermaid-validation` include contains invalid glob"),
        "{error:#}"
    );
}

#[test]
fn standalone_entrypoint_without_enabled_filesystem_rules_is_empty() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check-runner/empty");
    assert!(
        run_filesystem_rules(&root, Some(&root.join(".no-mistakes.yml")))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn supplied_tracked_entrypoint_rejects_invalid_prepared_catalog_options() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check-runner/empty");
    let config: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(
        "rules:\n  - rule: postgres-table-shape\n    scope: repository\n    options:\n      schemaCatalogPath: 42\n"
    ).unwrap();
    let error = run_filesystem_rules_with_config(&root, &config, &[]).unwrap_err();
    assert!(error.to_string().contains("schemaCatalogPath"), "{error:#}");
}
