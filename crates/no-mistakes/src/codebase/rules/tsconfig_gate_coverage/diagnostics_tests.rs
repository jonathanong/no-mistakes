#[test]
fn stale_allowlist_entries_point_to_the_fragment_entry() {
    let (root, config) = fixture_config();
    let rule = configured_rule(&config);
    let mut findings = stale_findings(rule);
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    super::annotate(&root, &config, rule, &sources, &mut findings);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes/rules/types.yml");
    assert_eq!(findings[0].line, 4);
}

#[test]
fn detached_rule_does_not_borrow_another_rules_authored_location() {
    let (root, config) = fixture_config();
    let rule = configured_rule(&config).clone();
    let mut findings = stale_findings(&rule);
    assert_eq!(findings.len(), 1);
    let original = findings.clone();
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    super::annotate(&root, &config, &rule, &sources, &mut findings);
    assert_eq!(findings, original);
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn missing_authored_allowlist_key_uses_fragment_with_first_line_fallback() {
    let (root, config) = fixture_config();
    let rule = configured_rule(&config);
    let mut findings = stale_findings(rule);
    assert_eq!(findings.len(), 1);
    findings[0].target = Some("unrecorded/tsconfig.json".to_string());
    let original_message = findings[0].message.clone();
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    super::annotate(&root, &config, rule, &sources, &mut findings);
    assert_eq!(findings[0].file, ".no-mistakes/rules/types.yml");
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].message, original_message);
    assert_eq!(sources.physical_read_count(), 1);
}

fn fixture_config() -> (std::path::PathBuf, crate::config::v2::NoMistakesConfig) {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/config-v2/split-provenance/fixture/rule-diagnostics"),
    );
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    (root, config)
}

fn configured_rule(
    config: &crate::config::v2::NoMistakesConfig,
) -> &crate::config::v2::schema::RuleDef {
    config
        .rules
        .iter()
        .find(|rule| rule.rule == super::super::RULE_ID)
        .unwrap()
}

fn stale_findings(
    rule: &crate::config::v2::schema::RuleDef,
) -> Vec<crate::codebase::rules::RuleFinding> {
    let options = rule.try_rule_options().unwrap();
    let empty = std::collections::BTreeSet::new();
    super::super::scan_application(
        &options,
        &empty,
        &empty,
        &empty,
        &empty,
        &empty,
        ".no-mistakes.yml",
    )
}
