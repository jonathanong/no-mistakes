#[test]
fn embedded_sql_rule_scans_keep_distinct_path_filters() {
    let mut config = NoMistakesConfig::default();
    for exclude in ["src/legacy/**", "src/other/**"] {
        config.rules.push(RuleDef {
            rule: "postgres-required-predicates".into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(&format!(
                "importSpecifier: '@example/db'\nexecutorNames: [pool]\nexclude: ['{exclude}']"
            ))
            .unwrap(),
            ..RuleDef::default()
        });
    }
    let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
    assign_prepared_embedded_sql_options(&config, &mut plan).unwrap();
    let scans = plan.embedded_sql_rule_ids.values().next().unwrap();
    assert_eq!(scans.len(), 2);
    let included = scans::EmbeddedSqlRuleScan::new(
        "postgres-required-predicates",
        &["src/current/**".into()],
        &["src/legacy/**".into()],
    )
    .unwrap();
    assert!(!included.paths.includes("src/legacy/old.ts"));
    assert!(included.paths.includes("src/current/now.ts"));
    assert!(!included.paths.includes("src/other/now.ts"));
}

#[test]
fn embedded_sql_rule_scan_rejects_an_invalid_glob() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "postgres-required-predicates".into(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("include: ['[']").unwrap(),
        ..RuleDef::default()
    });
    let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
    assert!(assign_prepared_embedded_sql_options(&config, &mut plan).is_err());
}
