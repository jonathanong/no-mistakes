use super::*;
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-calls/explicit-vitest-project/fixture"),
    )
}

fn fixture_findings(case: &str) -> anyhow::Result<Vec<crate::codebase::rules::RuleFinding>> {
    run_check(
        &fixture_root(),
        Some(&super::forbidden_calls_case(case)),
        None,
    )
}

#[test]
fn explicit_vitest_roots_still_reject_unknown_names() {
    let error = fixture_findings("explicit-vitest-unknown").unwrap_err();

    assert!(
        error.to_string().contains("names an unknown project"),
        "{error:#}"
    );
}

#[test]
fn prepared_catalog_merges_explicit_projects_without_runner_configs() {
    let root = fixture_root();
    let visible = crate::codebase::ts_source::discover_files(&root, &[]);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &visible);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let tsconfig_catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig, None);
    let config: NoMistakesConfig = serde_yaml::from_str(
        "tests:\n  vitest:\n    configs: missing.config.ts\n    projects:\n      custom:\n        include: [src/custom.spec.mts]",
    )
    .unwrap();
    let catalog = crate::codebase::rules::prepare_vitest_project_catalog(
        &root,
        &config,
        &snapshot,
        &tsconfig_catalog,
    );

    assert!(catalog.config_projects().unwrap().is_empty());
    let matched = catalog
        .matching_files(&root, &["custom".to_string()], &visible)
        .unwrap();
    assert_eq!(
        matched,
        vec![root.join("src/custom.spec.mts")],
        "{matched:?}"
    );
}

#[test]
fn vitest_root_errors_when_no_files_match() {
    // Skip runner discovery with a missing config, then use an include that
    // matches nothing so the empty-collection diagnostic stays distinct from I/O.
    let error = fixture_findings("explicit-vitest-no-files").unwrap_err();

    assert!(
        error.to_string().contains("Vitest root matched no files"),
        "{error:#}"
    );
}

#[test]
fn run_check_with_facts_prepares_the_vitest_catalog_for_vitest_roots() {
    let root = fixture_root();
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let graph_plan = crate::codebase::rules::try_canonical_graph_plan(&config)
        .unwrap()
        .expect("forbidden-calls requires a graph");
    let (graph, graph_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan(&root, graph_plan);
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        crate::codebase::ts_source::discover_files(&root, &[]),
        crate::codebase::check_facts::CheckFactPlan {
            graph,
            graph_context,
            ..Default::default()
        },
    );
    let findings =
        crate::codebase::rules::run_check_with_facts(&root, Some(&config_path), None, &shared)
            .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/custom.spec.mts");
}
