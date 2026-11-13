#[test]
fn count_skips_zero_and_records_observer_work() {
    super::count("postgres.mutable_sql_registry.producers", 0);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        super::count("postgres.mutable_sql_registry.producers", 2);
    });
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("postgres.mutable_sql_registry.producers")
            .copied(),
        Some(2)
    );
}

#[test]
fn check_application_rejects_bad_options_and_visit_failures() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/effort-low");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let rule_id = "postgres-no-unbounded-sql-identifier";
    let base = config.rule_applications(rule_id)[0].clone();
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root.clone(),
        Default::default(),
        Default::default(),
    );
    let facts = crate::codebase::check_facts::CheckFactMap::default();
    fn visit(
        _: &std::path::Path,
        _: &super::CompiledOptions,
        _: &super::DepGraph,
        _: &super::CheckFactMap,
    ) -> super::Result<Vec<super::RuleFinding>> {
        anyhow::bail!("blocked visit")
    }
    let mut application = base.clone();
    application.options =
        serde_yaml::from_str("include: 1\nimportSpecifier: \"@example/db\"\n").unwrap();
    let error =
        super::check_application(&root, &config, &application, &graph, &facts, rule_id, visit)
            .unwrap_err()
            .to_string();
    assert!(error.contains("invalid options"), "{error}");
    let mut application = base.clone();
    application.options =
        serde_yaml::from_str("exclude: ['[']\nimportSpecifier: \"@example/db\"\n").unwrap();
    let error =
        super::check_application(&root, &config, &application, &graph, &facts, rule_id, visit)
            .unwrap_err()
            .to_string();
    assert!(error.contains("exclude"), "{error}");
    let error = super::check_application(&root, &config, &base, &graph, &facts, rule_id, visit)
        .unwrap_err()
        .to_string();
    assert!(error.contains("blocked visit"), "{error}");
}
