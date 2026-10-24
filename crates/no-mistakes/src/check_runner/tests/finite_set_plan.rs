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

#[test]
fn runner_prepare_limits_call_fact_work_to_configured_sources() {
    use no_mistakes::codebase::analysis_session::AnalysisSession;
    use no_mistakes::codebase::check_facts::collect_check_facts_with_supplemental_call_sites;
    use no_mistakes::codebase::ts_source::facts::TsFactPlan;

    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/finite-set-consistency/call-literals/scoped-demand");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    let config_path = root.join("runner-demand.yml");
    let config = no_mistakes::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let selected = root.join("schedules.ts");
    let snapshot = no_mistakes::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let discovered = snapshot
        .paths_for(&root)
        .iter()
        .filter(|path| no_mistakes::codebase::dependencies::extract::is_indexable(path))
        .cloned()
        .collect::<Vec<_>>();
    let sources = snapshot.source_store_for(&root);
    assert_eq!(discovered.len(), 5, "fixture must expose all TS sources");

    let collect = |broad: bool| {
        let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
        let session = AnalysisSession::new(Some(Arc::clone(&observer)));
        let mut plan = CheckFactPlan {
            imports: true,
            symbols: true,
            graph: TsFactPlan {
                imports: true,
                call_sites: broad,
                ..Default::default()
            },
            ..Default::default()
        };
        let demand = super::prepare(&root, &config, &mut plan, broad, false).unwrap();
        assert_eq!(plan.graph.call_sites, broad);
        assert!(plan.call_site_paths.contains(&selected));
        assert_eq!(plan.call_site_paths.len(), 1);
        let primary = demand.primary_files(discovered.clone());
        assert_eq!(primary.len(), discovered.len());
        let supplemental = demand.supplemental_call_site_files(&primary, &[]);
        assert!(supplemental.is_empty());
        let facts = no_mistakes::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
            no_mistakes::ast::with_request_parse_cache(|| {
                collect_check_facts_with_supplemental_call_sites(
                    &session,
                    &root,
                    (primary, Vec::new()),
                    supplemental,
                    plan,
                    None,
                    Arc::clone(&sources),
                )
            })
        });
        assert_eq!(facts.files().len(), discovered.len());
        observer.snapshot().work
    };

    let scoped_work = collect(false);
    let broad_work = collect(true);
    assert_eq!(scoped_work.get("parse.files"), Some(&5), "{scoped_work:#?}");
    assert_eq!(broad_work.get("parse.files"), Some(&5), "{broad_work:#?}");
    // The runner's unrelated helper/config/registry/TSX sources must not pay
    // for call-site walks when only schedules.ts requests call literals.
    assert_eq!(scoped_work.get("ast.walks"), Some(&1), "{scoped_work:#?}");
    assert_eq!(broad_work.get("ast.walks"), Some(&5), "{broad_work:#?}");

    let results =
        crate::check_runner::run_all_with_evidence(root, Some(config_path), None, false, false)
            .unwrap();
    let findings = results
        .rules
        .iter()
        .filter(|finding| finding.rule == no_mistakes::codebase::rules::FINITE_SET_CONSISTENCY)
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("missing"));
}
