use no_mistakes::codebase::check_facts::CheckFactPlan;
use no_mistakes::config::v2::NoMistakesConfig;

#[test]
fn prepare_surfaces_invalid_finite_set_options() {
    let config: NoMistakesConfig = serde_yaml::from_str(
        r#"
rules:
  - rule: finite-set-consistency
    scope: repository
    options:
      sets: false
"#,
    )
    .unwrap();
    let mut plan = CheckFactPlan::default();

    let error = crate::check_runner::finite_set_plan::prepare(
        std::path::Path::new("/repo"),
        &config,
        &mut plan,
        false,
        false,
    )
    .err()
    .expect("invalid finite-set options must fail fact planning");

    assert!(error.to_string().contains("options.sets"), "{error:#}");
}

#[test]
fn explicit_runner_deadline_demand_does_not_admit_unrelated_primary_sources() {
    use no_mistakes::config::v2::schema::StringOrList;
    use std::sync::Arc;
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests/declared-deadlines/fixture"),
    );
    let configured = root.join("vitest.negative.ts");
    let unrelated = root.join("budgets.ts");
    let paths = vec![configured.clone(), unrelated.clone()];
    let sources = Arc::new(no_mistakes::codebase::ts_source::SourceStore::new(
        Arc::new(no_mistakes::codebase::ts_source::FileInventory::from_paths(
            &paths,
        )),
    ));
    let catalog = Arc::new(
        no_mistakes::codebase::ts_resolver::TsConfigCatalog::from_visible(
            &root,
            std::slice::from_ref(&root),
            &paths,
        ),
    );
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("vitest.negative.ts".into()));
    let mut plan = CheckFactPlan {
        integration_runner_configs: Some(Arc::new(
            no_mistakes::integration_tests::prepare_runner_configs_with_deadline_evidence(
                &root, &config, &paths, catalog, sources,
            ),
        )),
        ..Default::default()
    };
    let demand =
        crate::check_runner::finite_set_plan::prepare(&root, &config, &mut plan, false, false)
            .unwrap();
    assert!(demand.needs_shared_facts());
    assert_eq!(demand.primary_files(paths), vec![configured]);
    assert!(demand.supplemental_call_site_files(&[], &[]).is_empty());
    assert!(!plan.integration);
}
