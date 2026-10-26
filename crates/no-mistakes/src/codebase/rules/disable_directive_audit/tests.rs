use super::*;

pub(super) fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/disable-directive-audit"),
    )
}

#[test]
fn lexer_inventory_uses_actual_sql_comments_and_preserves_shadowed_directives() {
    let source =
        include_str!("../../../../../../fixtures/rules/disable-directive-audit/source.sql");
    let directives = disable_directives(source, true);
    assert_eq!(directives.len(), 4);
    assert_eq!(directives[0].directive, DisableDirective::File { line: 1 });
    assert_eq!(directives[1].directive, DisableDirective::File { line: 2 });
    assert_eq!(
        directives[2].directive,
        DisableDirective::NextLine { line: 3 }
    );
    assert_eq!(directives[3].directive, DisableDirective::Line { line: 5 });
}

#[test]
fn lexer_inventory_preserves_markdown_prose_quote_mode() {
    let source =
        include_str!("../../../../../../fixtures/rules/disable-directive-audit/source.markdown");
    let directives = disable_directives(source, false);
    assert_eq!(directives.len(), 2);
    assert_eq!(directives[0].directive, DisableDirective::Line { line: 1 });
}

#[test]
fn disabled_only_scope_skips_options_validation_and_reuses_frozen_sources() {
    let root = root();
    let files = vec![root.join("source.sql")];
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(
        "../../../../../../fixtures/rules/disable-directive-audit/disabled.yml"
    ))
    .unwrap();
    let sources = super::super::source_store_for_files(&files);
    sources.read_path(&files[0]).unwrap();
    let findings = audit(&root, &config, &files, &sources, &[]).unwrap();
    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(findings.len(), 4);
    assert!(findings
        .iter()
        .all(|item| item.rule == "invalid-disable-directive" && item.message.contains("disabled")));
}

#[test]
fn provenance_accounting_uses_source_file_instead_of_public_finding_target() {
    let root = root();
    let files = vec![root.join("source.sql")];
    let mut config: NoMistakesConfig = serde_yaml::from_str(include_str!(
        "../../../../../../fixtures/rules/disable-directive-audit/disabled.yml"
    ))
    .unwrap();
    config.rules[0].enabled = true;
    config.rules[0].options = serde_yaml::Value::Null;
    let sources = super::super::source_store_for_files(&files);
    let consumed = SuppressedFinding {
        domain: "rules".into(),
        rule: "postgres-no-offset".into(),
        file: "public-target.ts".into(),
        source_file: root.join("source.sql").to_string_lossy().into_owned(),
        line: Some(42),
        reason: "actual violation at helper source".into(),
        directive: SuppressionDirective {
            kind: SuppressionDirectiveKind::File,
            line: 1,
        },
    };
    let findings = audit(&root, &config, &files, &sources, &[consumed]).unwrap();
    assert_eq!(findings.len(), 3);
    assert!(findings.iter().all(|item| item.line != 1));
}

#[test]
fn known_rule_catalog_includes_core_rules_without_public_id_aliases() {
    assert!(known_rule(
        super::super::production_exports_test_only::RULE_ID
    ));
    // This core rule intentionally has no entry in ids.rs or the filesystem registry.
    assert!(known_rule(
        super::super::coverage_producer_assertion_owner::RULE_ID
    ));
    assert!(known_rule(
        super::super::postgres_require_fk_on_delete::RULE_ID
    ));
    assert!(!known_rule("typo-configured-as-a-rule"));
}

#[test]
fn invalid_utf8_retained_bytes_share_used_and_unused_directive_semantics() {
    let root = root();
    let files = vec![root.join("invalid-utf8.sql")];
    let mut config: NoMistakesConfig = serde_yaml::from_str(include_str!(
        "../../../../../../fixtures/rules/disable-directive-audit/disabled.yml"
    ))
    .unwrap();
    config.rules[0].enabled = true;
    config.rules[0].options = serde_yaml::Value::Null;
    config.rules[0].include = vec!["invalid-utf8.sql".into()];
    let sources = super::super::source_store_for_files(&files);
    assert!(sources.read_path(&files[0]).is_err());
    let prepared = PreparedDisableDirectives::prepare(&root, &config, &files, &sources).unwrap();
    assert_eq!(prepared.findings(&[]).len(), 2);
    // A required-source finding is hidden by the file directive, while the
    // shadowed next-line directive still suppressed nothing.
    let mut findings = vec![RuleFinding {
        rule: "postgres-no-offset".into(),
        file: "invalid-utf8.sql".into(),
        line: 3,
        message: "cannot read source as UTF-8".into(),
        import: None,
        target: None,
    }];
    let suppressed = super::super::suppress_domain_findings_with_sources(
        &root,
        &mut findings,
        &sources,
        |finding| super::super::SuppressionTarget {
            domain: "rules",
            rule: &finding.rule,
            file: &finding.file,
            line: Some(finding.line),
            reason: &finding.message,
            identity: None,
        },
    );
    assert!(findings.is_empty());
    assert_eq!(suppressed.len(), 1);
    let stale = prepared.findings(&suppressed);
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].line, 2);
    assert_eq!(stale[0].rule, "unused-disable-directive");
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn prepared_comments_finalize_after_source_owner_is_dropped() {
    let root = root();
    let files = vec![root.join("source.sql")];
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(
        "../../../../../../fixtures/rules/disable-directive-audit/disabled.yml"
    ))
    .unwrap();
    let sources = super::super::source_store_for_files(&files);
    let prepared = PreparedDisableDirectives::prepare(&root, &config, &files, &sources).unwrap();
    assert_eq!(sources.physical_read_count(), 1);
    let first = prepared.findings(&[]);
    assert_eq!(sources.physical_read_count(), 1);
    // Finalization owns only comment metadata, so it cannot reread even a cached source.
    drop(sources);
    let repeated = prepared.findings(&[]);
    assert_eq!(first.len(), 4);
    assert_eq!(
        serde_json::to_value(first).unwrap(),
        serde_json::to_value(repeated).unwrap()
    );
}

#[test]
fn ignored_audit_preparation_does_not_read_sources() {
    let root = root();
    let files = vec![root.join("source.sql")];
    let mut config: NoMistakesConfig = serde_yaml::from_str(include_str!(
        "../../../../../../fixtures/rules/disable-directive-audit/disabled.yml"
    ))
    .unwrap();
    config.report_unused_disable_directives = DisableDirectivePolicy::Ignore;
    let sources = super::super::source_store_for_files(&files);
    let prepared = PreparedDisableDirectives::prepare(&root, &config, &files, &sources).unwrap();
    assert_eq!(sources.physical_read_count(), 0);
    assert!(prepared.findings(&[]).is_empty());
    assert_eq!(sources.physical_read_count(), 0);
}
