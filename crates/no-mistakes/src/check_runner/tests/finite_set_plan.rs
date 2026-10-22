use no_mistakes::codebase::check_facts::CheckFactPlan;
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::PathBuf;
use std::sync::Arc;

type CompanionFixtureInputs = (
    PathBuf,
    NoMistakesConfig,
    Vec<PathBuf>,
    Arc<no_mistakes::codebase::ts_source::SourceStore>,
);

fn companion_fixture() -> CompanionFixtureInputs {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/performance/required-companion-imports"),
    );
    let config = no_mistakes::config::v2::load_v2_config(&root, None).unwrap();
    let snapshot = no_mistakes::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let files = snapshot.paths_for(&root).as_ref().clone();
    let sources = snapshot.source_store_for(&root);
    (root, config, files, sources)
}

#[test]
fn finite_set_preparation_keeps_graph_call_sites_independent() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped"),
    );
    let config = no_mistakes::config::v2::load_v2_config(&root, None).unwrap();
    let selected = root.join("generated/schedules.test.mts");
    let unrelated = root.join("registry.mts");
    let mut plan = CheckFactPlan::default();
    let demand = super::prepare(&root, &config, &mut plan, false, false).unwrap();
    assert!(Arc::ptr_eq(&demand.call_site_files, &plan.call_site_paths));
    assert!(plan.call_site_paths.contains(&selected));
    assert!(!plan.call_site_paths.contains(&unrelated));
    assert!(!plan.graph.call_sites);
    assert_eq!(
        demand.supplemental_call_site_files(&[], &[]),
        vec![selected]
    );

    plan.graph.call_sites = true;
    let _ = super::prepare(&root, &config, &mut plan, true, false).unwrap();
    assert!(
        plan.graph.call_sites,
        "graph-owned all-file demand must survive"
    );
}

#[test]
fn production_only_demand_never_copies_companion_membership() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/dependency-checks");
    let config = no_mistakes::config::v2::load_v2_config(&root, None).unwrap();
    let snapshot = no_mistakes::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let files = snapshot.paths_for(&root).as_ref().clone();
    let paths = no_mistakes::codebase::rules::production_dependency_declarations::required_import_fact_files(
        &root, &config, &files,
    ).unwrap();
    assert!(!paths.is_empty());
    let mut plan = CheckFactPlan {
        production_import_paths: Arc::clone(&paths),
        ..Default::default()
    };
    let mut demand = super::prepare(&root, &config, &mut plan, false, false).unwrap();
    assert!(demand.needs_shared_facts());
    assert!(demand.companion_import_files.is_empty());
    demand.include_companion_plan(&plan);
    assert!(demand.companion_import_files.is_empty());
    assert_eq!(demand.primary_files(files.clone()), files);
    assert!(Arc::ptr_eq(&paths, &plan.production_import_paths));
}

#[test]
fn companion_demand_does_not_copy_membership_for_a_complete_primary_scope() {
    let (root, config, files, sources) = companion_fixture();
    let mut plan = CheckFactPlan {
        symbols: true,
        ..Default::default()
    };
    let mut demand = super::prepare(&root, &config, &mut plan, false, false).unwrap();
    no_mistakes::codebase::rules::required_companion_imports::configure_fact_plan(
        &root, &config, &files, &sources, &mut plan,
    )
    .unwrap();
    assert_eq!(plan.production_import_paths.len(), 4);
    demand.include_companion_plan(&plan);
    assert!(demand.has_companion_plan);
    assert!(demand.companion_import_files.is_empty());
    assert_eq!(demand.primary_files(files.clone()), files);
    // Preparing an already-unioned plan has the same complete-scope behavior.
    let demand = super::prepare(&root, &config, &mut plan, false, false).unwrap();
    assert!(demand.has_companion_plan);
    assert!(demand.companion_import_files.is_empty());
}

#[test]
fn companion_only_demand_retains_its_selected_primary_scope() {
    let (root, config, files, sources) = companion_fixture();
    let mut plan = CheckFactPlan::default();
    let mut demand = super::prepare(&root, &config, &mut plan, false, false).unwrap();
    assert!(!demand.needs_shared_facts());
    no_mistakes::codebase::rules::required_companion_imports::configure_fact_plan(
        &root, &config, &files, &sources, &mut plan,
    )
    .unwrap();
    demand.include_companion_plan(&plan);
    assert!(demand.needs_shared_facts());
    assert_eq!(demand.companion_import_files.len(), 4);
    let expected = (0..4)
        .map(|index| root.join(format!("stories/group{index}.stories.ts")))
        .collect::<Vec<_>>();
    assert_eq!(demand.primary_files(files), expected);
    assert_eq!(plan.production_import_paths.len(), 4);
}

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
