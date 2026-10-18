#[test]
fn invalid_schema_and_statement_options_stop_request_planning() {
    for rule in ["postgres-identifier-length", "postgres-sql-shape-policy"] {
        let config = NoMistakesConfig {
            rules: vec![RuleDef {
                rule: rule.into(),
                scope: Some(RuleScope::Repository),
                options: serde_yaml::from_str("sqlInclude: false\nexecutorNames: []").unwrap(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(
            configure_prepared_postgres_plan(&config, &mut Default::default())
                .unwrap_err()
                .to_string()
                .contains("sqlInclude")
        );
    }
}

#[test]
fn bounded_rule_alone_requests_bounds_and_standalone_preparation_keeps_them() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture/prepared-demand");
    let file = root.join("queries.sql");
    let files = vec![file.clone()];
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &files);
    let sources = snapshot.source_store_for(&root);
    for (rule, expected) in [
        ("postgres-required-predicates", false),
        ("postgres-sql-shape-policy", false),
        ("postgres-bounded-statements", true),
    ] {
        let config = rule_config(rule);
        let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
        configure_prepared_postgres_plan(&config, &mut plan).unwrap();
        assert!(plan.postgres_dml);
        assert_eq!(plan.postgres_bounds, expected, "{rule}");
        let prepared = prepare_rule_sql_facts(
            &root,
            &files,
            std::sync::Arc::clone(&sources),
            &config,
            &[rule],
        )
        .unwrap();
        let statements = prepared.postgres.statements(&file, None).unwrap();
        assert_eq!(
            statements[0].bounds.len(),
            if expected { 3 } else { 0 },
            "{rule}"
        );
    }
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn every_embedded_rule_requires_explicit_executor_selection() {
    use crate::codebase::postgres::extract_embedded_sql_from_source;
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/string-literal.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let mut ids = PREPARED_EMBEDDED_SQL_RULE_IDS.to_vec();
    ids.extend([
        "postgres-idempotent-insert",
        "postgres-require-query-annotation",
    ]);
    for rule_id in ids {
        for (yaml, expected_calls) in [
            ("executorNames: []", 0),
            ("importSpecifier: '@example/db'", 1),
            ("executorNames: [query]", 1),
            ("importSpecifier: '@other/db'", 0),
        ] {
            let mut config = NoMistakesConfig::default();
            config.rules.push(RuleDef {
                rule: rule_id.to_string(),
                scope: Some(RuleScope::Repository),
                options: serde_yaml::from_str(yaml).unwrap(),
                ..RuleDef::default()
            });
            let profiles = configured_embedded_sql_options(&config, &[rule_id]).unwrap();
            // SQL-file-only statement policy has no executor projection at all.
            let calls: usize = profiles
                .iter()
                .map(|profile| {
                    extract_embedded_sql_from_source(&path, &source, profile)
                        .calls
                        .len()
                })
                .sum();
            assert_eq!(calls, expected_calls, "{rule_id}: {yaml}");
        }
    }
}
