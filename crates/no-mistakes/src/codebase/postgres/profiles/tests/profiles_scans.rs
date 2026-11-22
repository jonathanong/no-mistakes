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

#[test]
fn annotation_scan_restricts_explicit_roots_and_leaves_framework_inference_to_dispatch() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/check-facts/family-scope-nextjs")
        .canonicalize()
        .unwrap();
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "web".into(),
        crate::config::v2::schema::Project {
            type_: Some(crate::config::v2::schema::ProjectType::Nextjs),
            ..Default::default()
        },
    );
    config.rules.push(RuleDef {
        rule: "postgres-require-query-annotation".into(),
        projects: vec!["web".into()],
        options: serde_yaml::from_str(
            "effort: max\nimportSpecifier: '@app/db'\nexecutorNames: [query]",
        )
        .unwrap(),
        ..Default::default()
    });
    let outside = "backend/provider.ts";
    for (project_root, allowed) in [(None, true), (Some("web".to_owned()), false)] {
        config.projects.get_mut("web").unwrap().root = project_root;
        let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
        configure_prepared_postgres_plan(&root, &config, &mut plan).unwrap();
        let scans = plan.query_annotation_scans.as_ref().unwrap();
        let [scan] = scans.values().next().unwrap().as_slice() else {
            panic!("one annotation application");
        };
        assert_eq!(scan.annotation_unrestricted(), allowed);
        assert_eq!(
            scan.includes_annotation(&root.join(outside), outside),
            allowed
        );
        assert!(scan.includes_annotation(&root.join("web/src/selected.ts"), "web/src/selected.ts"));
    }
}
