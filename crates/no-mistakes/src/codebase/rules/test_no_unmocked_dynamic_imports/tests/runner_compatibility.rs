use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::ts_resolver::TsConfigCatalog;
use crate::integration_tests::runner_config::prepare_dynamic_configs;
use crate::integration_tests::types::Framework;

fn facts(root: &Path) -> crate::codebase::check_facts::CheckFactMap {
    crate::codebase::check_facts::collect_check_facts(
        root,
        crate::codebase::ts_source::discover_files(root, &[]),
        crate::codebase::check_facts::CheckFactPlan {
            imports: true,
            dynamic_imports: true,
            source: true,
            ..Default::default()
        },
    )
}

#[test]
fn partial_runner_facts_are_supplemented_by_framework_and_retained() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-project-setups",
        ),
    );
    let config = crate::config::v2::load_v2_config(
        &root,
        Some(&root.join(".no-mistakes-shared-runner-path.yml")),
    )
    .unwrap();
    let seed = crate::config::v2::load_v2_config(
        &root,
        Some(&root.join(".no-mistakes-shared-runner-seed.yml")),
    )
    .unwrap();
    let mut shared = facts(&root);
    let sources = crate::codebase::rules::source_store_for_files(shared.files());
    let catalog = Arc::new(TsConfigCatalog::forced(&root, Default::default(), None));
    let runner = prepare_dynamic_configs(
        &root,
        &seed,
        shared.files(),
        Arc::clone(&catalog),
        Arc::clone(&sources),
    );
    shared.integration_runner_configs = runner.parse_all().unwrap().file_facts().clone();
    let path = root.join("shared-runner.config.mts");
    let original = shared.integration_runner_configs[&path].projects_for(Framework::Playwright);
    assert!(original.is_some());
    assert!(shared.integration_runner_configs[&path]
        .vitest_projects()
        .is_none());
    let session = AnalysisSession::new(Some(crate::diagnostics::InvocationObserver::new(true)));
    let supplemented = super::super::with_facts::supplement_runner_facts(
        &root, &config, &catalog, &shared, &session, &sources,
    )
    .expect("a present path without Vitest results is incomplete");
    let merged = &supplemented.integration_runner_configs[&path];
    let retained = merged.projects_for(Framework::Playwright).unwrap().unwrap();
    let original = original.unwrap().unwrap();
    assert_eq!(retained.len(), original.len());
    assert_eq!(retained[0].policy_name, original[0].policy_name);
    assert_eq!(retained[0].include, original[0].include);
    assert!(merged.vitest_projects().unwrap().is_ok());
    let reads = sources.physical_read_count();
    assert!(super::super::with_facts::supplement_runner_facts(
        &root,
        &config,
        &catalog,
        &supplemented,
        &session,
        &sources,
    )
    .is_none());
    assert_eq!(sources.physical_read_count(), reads);
    assert_eq!(session.work_snapshot().parse_attempts[&path], 1);
    for findings in [
        check_with_facts(&root, &config, None, &shared).unwrap(),
        crate::codebase::rules::run_check_with_facts(
            &root,
            Some(&root.join(".no-mistakes-shared-runner-path.yml")),
            None,
            &shared,
        )
        .unwrap(),
    ] {
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.file.as_str())
                .collect::<Vec<_>>(),
            vec!["web/covered.test.mts"]
        );
    }
}

#[test]
fn compatibility_runner_catalog_seeds_explicit_config_aliases() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-runner-aliases",
        ),
    );
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let shared = facts(&root);
    assert!(check_with_facts(&root, &config, None, &shared)
        .unwrap()
        .is_empty());
    assert!(
        crate::codebase::rules::run_check_with_facts(&root, None, None, &shared)
            .unwrap()
            .is_empty()
    );
}
