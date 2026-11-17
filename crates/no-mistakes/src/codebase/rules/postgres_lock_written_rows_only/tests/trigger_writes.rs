use super::*;

#[test]
fn counts_only_matching_parsed_trigger_write_chains() {
    let root = fixture("trigger-writes");
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-lock-written-rows-only/configs/trigger-writes.yml"
    )))
    .unwrap();
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings.len(), 9, "{findings:?}");
    let lines: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [29, 35, 41, 47, 53, 59, 65, 71, 77]);
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some(CHECKS[0])));
}

#[test]
fn omitted_catalog_keeps_trigger_relations_unproven() {
    let findings = check(&fixture("trigger-writes"), &config()).unwrap();
    assert_eq!(findings.len(), 13, "{findings:?}");
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some(CHECKS[0])));
}

#[test]
fn trigger_depth_bound_retains_reports_beyond_proven_relations() {
    let root = fixture("trigger-depth");
    let mut config = config();
    config.rules[0].options["schemaCatalogPath"] = serde_yaml::to_value("schema.json").unwrap();
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 11);
    assert_eq!(findings[0].target.as_deref(), Some(CHECKS[0]));
}

#[test]
fn upsert_update_fires_only_matching_update_triggers() {
    let root = fixture("trigger-upsert");
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-lock-written-rows-only/configs/trigger-writes.yml"
    )))
    .unwrap();
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [11, 17]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some(CHECKS[0])));
}

#[test]
fn dynamic_setting_names_disable_only_trigger_evidence() {
    let root = fixture("trigger-dynamic-setting");
    let mut config = config();
    config.rules[0].options["schemaCatalogPath"] = serde_yaml::to_value("schema.json").unwrap();
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [6, 27]
    );
    assert_eq!(findings[0].target.as_deref(), Some(CHECKS[0]));
}

#[test]
fn excluded_catalog_keeps_trigger_relations_unproven() {
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-lock-written-rows-only/configs/trigger-catalog-excluded.yml"
    )))
    .unwrap();
    let findings = check(&fixture("trigger-writes"), &config).unwrap();
    assert_eq!(findings.len(), 13, "{findings:?}");
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some(CHECKS[0])));
}

#[test]
fn missing_trigger_catalog_reports_the_configured_path() {
    let mut config = config();
    config.rules[0].options["schemaCatalogPath"] = serde_yaml::to_value("absent.json").unwrap();
    let error = check(&fixture("trigger-writes"), &config).unwrap_err();
    assert!(error.to_string().contains("absent.json"), "{error}");
}
