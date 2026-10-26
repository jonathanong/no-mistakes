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

#[test]
fn test_state_profile_reuses_only_explicit_test_and_helper_scope() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "postgres-no-test-state-mutation".into(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(
            "importSpecifier: '@example/db'\ntestGlobs: ['specs/**/*.ts']\nhelperRoots: ['support/**']",
        )
        .unwrap(),
        ..RuleDef::default()
    });
    let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
    assign_prepared_embedded_sql_options(&config, &mut plan).unwrap();
    configure_prepared_postgres_plan(Path::new("."), &config, &mut plan).unwrap();
    assert!(plan.embedded_sql && plan.postgres_dml && plan.postgres_general_locations);
    assert!(!plan.postgres_schema);
    assert!(plan.postgres_sql_include.is_empty());
    let scans = plan.embedded_sql_rule_ids.values().next().unwrap();
    assert!(scans[0].paths.includes("specs/database/test.ts"));
    assert!(scans[0].paths.includes("support/sql.ts"));
    assert!(!scans[0].paths.includes("src/production.ts"));
    assert!(!scans[0].paths.includes("migrations/001.sql"));
}
