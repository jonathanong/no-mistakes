use super::*;

fn assertion_config() -> (PathBuf, NoMistakesConfig) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/identifier-assertion-config");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("profiles.yml"))).unwrap();
    (root, config)
}

#[test]
fn identifier_assertion_profiles_are_rule_local_and_keep_inherited_executors() {
    let (_, config) = assertion_config();
    let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
    assign_prepared_embedded_sql_options(&config, &mut plan).unwrap();
    assert_eq!(plan.embedded_sql_options.len(), 2);
    for (profile, scans) in &plan.embedded_sql_rule_ids {
        assert_eq!(profile.import_specifier, "@example/db");
        assert_eq!(profile.executor_names, ["custom", "read", "write"]);
        assert_eq!(profile.exclude_executor_names, ["query"]);
        if profile.sql_identifier_assertions.is_empty() {
            assert!(profile.raw_sql_tags.is_empty());
            assert_eq!(scans.len(), 1);
            assert_eq!(scans[0].rule_id, "postgres-no-offset");
        } else {
            assert_eq!(profile.raw_sql_tags.len(), 1);
            assert_eq!(profile.raw_sql_tags[0].module, "@example/sql");
            assert_eq!(profile.raw_sql_tags[0].name, "raw");
            assert_eq!(profile.sql_identifier_assertions.len(), 1);
            let assertion = &profile.sql_identifier_assertions[0];
            assert_eq!(assertion.module, "@example/identifiers");
            assert_eq!(assertion.name, "assertIdentifier");
            assert_eq!(assertion.allowlist_argument, 1);
            assert_eq!(scans.len(), 2);
            assert!(scans
                .iter()
                .all(|scan| scan.rule_id != "postgres-no-offset"));
        }
    }
}

#[test]
fn low_identifier_profiles_preserve_rule_local_assertion_identity() {
    let (root, mut config) = assertion_config();
    for rule in &mut config.rules {
        rule.resolved_analysis = Some(crate::config::v2::EffectiveRuleAnalysis {
            effort: crate::config::v2::Effort::Low,
            ..Default::default()
        });
    }
    let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
    assign_prepared_embedded_sql_options(&config, &mut plan).unwrap();
    configure_prepared_postgres_plan(&root, &config, &mut plan).unwrap();
    assert_eq!(plan.sql_identifier_profiles.len(), 2);
    for (profile, scans) in &plan.sql_identifier_profiles {
        assert_eq!(scans.len(), 1);
        assert_eq!(profile.import_specifier, "@example/db");
        assert_eq!(profile.executor_names, ["custom", "read", "write"]);
    }
    let state = configured_embedded_sql_options(
        &assertion_config().1,
        &["postgres-no-test-state-mutation"],
    )
    .unwrap();
    assert!(plan.sql_identifier_profiles.contains_key(&state[0]));
    let offset = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_extra_executor_names(Some(&["custom".into()]))
        .with_exclude_executor_names(&["query".into()]);
    assert!(plan.sql_identifier_profiles.contains_key(&offset));
}
