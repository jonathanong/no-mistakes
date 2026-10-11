use super::*;
use crate::config::v2::schema::StringOrList;

fn prepare_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<prepared::PreparedConfig> {
    prepared::prepare_inner(root, config, files, sources, None)
}

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports/fixture"),
    )
}

#[test]
fn named_project_helper_does_not_supply_an_unscoped_setup_fallback() {
    use crate::codebase::ts_resolver::TsConfigCatalog;
    use std::sync::Arc;

    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/selected-vitest-excludes/fixture"),
    );
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let runner = crate::integration_tests::runner_config::prepare_dynamic_configs(
        &root,
        &config,
        &files,
        Arc::new(TsConfigCatalog::forced(&root, Default::default(), None)),
        Arc::clone(&sources),
    );
    let parsed = runner.parse_all().unwrap();
    let prepared =
        prepare_with_runner_facts(&root, &config, &files, &sources, parsed.file_facts()).unwrap();

    // The discovered `vitest.projects.mts` exports a named array used by an
    // alternate config. A's raw setup must not cover B's execution here.
    assert_eq!(prepared.setup_data().len(), 2);
    assert_eq!(
        setup_contexts_for_test_precomputed(
            "shared/setup.test.mts",
            prepared.setup_data(),
            SelectedRunners::for_config(&config),
        ),
        vec![vec![root.join("setup/mock.mts")], Vec::new()]
    );
}

#[test]
fn prepared_config_globs_only_expand_aggregate_candidates() {
    let root = fixture();
    let selected_config = root.join("jest.config.cjs");
    let mut config = NoMistakesConfig::default();
    config.tests.jest.configs = Some(StringOrList::One("jest.config.*".to_string()));

    // `jest.config.mjs` also exists and matches the configured glob. Omitting it
    // from the aggregate candidates must keep its matcher and setup files out.
    let sources =
        crate::codebase::rules::source_store_for_files(std::slice::from_ref(&selected_config));
    let prepared = prepare_from_visible(&root, &config, &[selected_config], &sources).unwrap();

    assert!(prepared
        .test_filter()
        .is_match("tests/example.cjs-spec.mts"));
    assert!(!prepared.test_filter().is_match("tests/example.spec.mts"));
    assert_eq!(prepared.setup_data().len(), 1);
    assert!(prepared.setup_data()[0].setup_files.is_empty());
}

#[test]
fn aggregate_rule_uses_prepared_config_without_standalone_discovery() {
    let source = include_str!("../with_facts.rs");

    assert!(source.contains("config::prepare_with_runner_facts("));
    assert!(!source.contains("config::test_filter("));
    assert!(!source.contains("config::precompute_setup_data("));
    assert!(!source.contains("discover_files("));
}

#[test]
fn prepared_config_reports_missing_visible_config_source() {
    let root = fixture();
    let missing = root.join("missing-visible-vitest.config.ts");
    assert!(
        !missing.exists(),
        "the fixture intentionally has no config file"
    );
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One(
        "missing-visible-vitest.config.*".to_string(),
    ));

    // The request inventory is authoritative: a config listed there without a
    // readable source must report an error instead of falling back to disk.
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&missing));
    let error = prepare_from_visible(&root, &config, &[missing], &sources)
        .err()
        .expect("missing prepared source must be reported");

    assert!(error.to_string().contains("failed to read"));
}

#[test]
fn pass4b_prepared_setup_files_drop_ignored_candidate_and_keep_visible_fallback() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4b-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let visible = crate::codebase::ts_source::discover_visible_paths(&root);
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("dynamic/vitest.config.ts".to_string()));

    let sources = crate::codebase::rules::source_store_for_files(&visible);
    let prepared = prepare_from_visible(&root, &config, &visible, &sources).unwrap();

    assert_eq!(
        prepared.setup_data()[0].setup_files,
        vec![root.join("dynamic/setup.ts")]
    );
}

#[test]
fn runner_setup_projection_requires_prepared_vitest_projects() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-project-setups",
        ),
    );
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes-inherit.yml")))
            .unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let error = prepare_with_runner_facts(&root, &config, &files, &sources, &Default::default())
        .err()
        .expect("the runner projection must be supplied by its owner");
    assert!(error
        .to_string()
        .contains("missing prepared Vitest config projects"));
}

#[test]
fn playwright_targets_reuse_prepared_runner_facts_and_ignore_unselected_vitest() {
    use crate::codebase::analysis_session::AnalysisSession;
    use crate::codebase::ts_resolver::TsConfigCatalog;
    use std::sync::Arc;
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-project-setups",
        ),
    );
    let config = crate::config::v2::load_v2_config(
        &root,
        Some(&root.join(".no-mistakes-playwright-config.yml")),
    )
    .unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let runner = crate::integration_tests::runner_config::prepare_dynamic_configs(
        &root,
        &config,
        &files,
        Arc::new(TsConfigCatalog::forced(&root, Default::default(), None)),
        Arc::clone(&sources),
    );
    let path = root.join("playwright.target.config.mts");
    assert_eq!(runner.paths().collect::<Vec<_>>(), vec![&path]);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(observer));
    let facts = std::collections::BTreeMap::from([(
        path.clone(),
        runner
            .parse_path_for_facts_with_session(&session, &path)
            .unwrap(),
    )]);
    let reads = sources.physical_read_count();
    for _ in 0..2 {
        let prepared = prepare_with_runner_facts(&root, &config, &files, &sources, &facts).unwrap();
        assert!(prepared.test_filter().is_match("web/covered.test.mts"));
        assert_eq!(prepared.setup_data().len(), 1);
        assert_eq!(
            setup_contexts_for_test_precomputed(
                "web/covered.test.mts",
                prepared.setup_data(),
                SelectedRunners::for_config(&config),
            ),
            vec![Vec::<PathBuf>::new()]
        );
    }
    assert_eq!(sources.physical_read_count(), reads);
    assert_eq!(session.work_snapshot().parse_attempts[&path], 1);
    let missing = prepare_with_runner_facts(&root, &config, &files, &sources, &Default::default())
        .err()
        .expect("the project projection must come from the request owner");
    assert!(missing
        .to_string()
        .contains("missing prepared playwright config projects"));
    let failures = std::collections::BTreeMap::from([(
        path.clone(),
        runner
            .parse_error(&path, "prepared fixture failure".to_string())
            .unwrap(),
    )]);
    let error = prepare_with_runner_facts(&root, &config, &files, &sources, &failures)
        .err()
        .expect("prepared errors must be authoritative over the valid disk config");
    assert_eq!(error.to_string(), "prepared fixture failure");
}
