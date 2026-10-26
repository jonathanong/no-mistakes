use super::*;

fn root() -> PathBuf {
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
    // This core rule intentionally has no entry in ids.rs or the filesystem registry.
    assert!(known_rule(
        super::super::coverage_producer_assertion_owner::RULE_ID
    ));
    assert!(known_rule(
        super::super::postgres_require_fk_on_delete::RULE_ID
    ));
    assert!(!known_rule("typo-configured-as-a-rule"));
}
