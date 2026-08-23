fn selector_work_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/nextjs-selectors/selector-covered/fixture"),
    )
}

fn selector_playwright_plan(root: &Path) -> crate::codebase::check_facts::PlaywrightFactPlan {
    let settings =
        crate::playwright::config::test_support::load_settings(root, None, &[], None).unwrap();
    let playwright_configs = crate::playwright::playwright_config::load_many(
        root,
        &settings.playwright_configs,
        settings.project.as_deref(),
    )
    .unwrap();
    let mut test_id_attributes_by_path = std::collections::HashMap::new();
    for test_file in
        crate::playwright::test_support::discover_test_files(root, &settings, &playwright_configs)
            .unwrap()
    {
        let attributes = test_file.test_id_attributes();
        test_id_attributes_by_path.insert(test_file.path, attributes);
    }
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(root);
    crate::codebase::check_facts::PlaywrightFactPlan::from_settings(
        root,
        settings,
        test_id_attributes_by_path,
        false,
        &snapshot,
    )
    .unwrap()
}

/// Prepared Playwright facts must supply selector edges without a second
/// graph build or another parse pass during emission.
#[test]
fn prepared_selector_edges_do_not_reparse_or_rebuild_graph() {
    let root = selector_work_fixture();
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts_with_graph_files_and_playwright(
        &root,
        all_files.clone(),
        all_files.clone(),
        crate::codebase::check_facts::CheckFactPlan::default(),
        Some(selector_playwright_plan(&root)),
    );
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..TsConfig::default()
    };
    let graph_files = facts.graph_file_universe().to_vec();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let graph = crate::diagnostics::with_observer(Some(observer.clone()), || {
        DepGraph::build_with_plan_file_list_and_check_facts(
            &root,
            &tsconfig,
            GraphBuildPlan {
                playwright_selectors: true,
                ..GraphBuildPlan::default()
            },
            graph_files,
            &facts,
        )
        .expect("selector graph builds from prepared facts")
    });

    let work = observer.snapshot().work;
    assert_eq!(work["graph.builds"], 1, "{work:#?}");
    assert_eq!(
        work.get("parse.files").copied().unwrap_or_default(),
        0,
        "selector edge emission must not parse again when Playwright facts are prepared: {work:#?}"
    );
    let spec = NodeId::file(root.join("tests/e2e/app.spec.ts"));
    let covered = NodeId::file(root.join("web/app/page.tsx"));
    let dependencies = graph.deps_of(&[spec], None, Some(&HashSet::from([EdgeKind::Selector])));
    assert!(
        dependencies.iter().any(|entry| entry.node == covered),
        "prepared facts must still emit the selector edge, got {dependencies:?}"
    );
}
