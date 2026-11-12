use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_sources_and_session, CheckFactPlan,
};
use crate::codebase::ts_source::VisiblePathSnapshot;

#[test]
fn unrelated_shared_failures_do_not_poison_exact_filesystem_demands() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check/filesystem-prerequisites"),
    );
    let snapshot = VisiblePathSnapshot::new(&root);
    let files = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    for (config_name, consumer, failed) in [
        ("finite-raw.yml", FINITE_SET_CONSISTENCY, false),
        ("finite-needed.yml", FINITE_SET_CONSISTENCY, true),
        ("companion-excluded.yml", REQUIRED_COMPANION_IMPORTS, false),
        ("companion-noop.yml", REQUIRED_COMPANION_IMPORTS, false),
        ("companion-needed.yml", REQUIRED_COMPANION_IMPORTS, true),
        ("lint-filter.yml", FORBIDDEN_LINT_SUPPRESSIONS, false),
        ("lint-needed.yml", FORBIDDEN_LINT_SUPPRESSIONS, true),
        (
            "production-workspace.yml",
            PRODUCTION_DEPENDENCY_DECLARATIONS,
            false,
        ),
        (
            "production-needed.yml",
            PRODUCTION_DEPENDENCY_DECLARATIONS,
            true,
        ),
    ] {
        let config = crate::config::v2::load_v2_config_from_visible(
            &root,
            Some(&root.join(config_name)),
            &files,
        )
        .unwrap();
        let session = AnalysisSession::disabled();
        session.enable_partial_checks();
        let mut plan = CheckFactPlan {
            graph: crate::codebase::ts_source::facts::TsFactPlan::imports(),
            ..Default::default()
        };
        crate::codebase::rules::required_companion_imports::configure_fact_plan(
            &root, &config, &files, &sources, &mut plan,
        )
        .unwrap();
        // A sibling request's call sites must never define the finite-set scope.
        plan.require_call_site_paths(&[root.join("broken.ts")]);
        let facts = collect_check_facts_with_graph_files_playwright_sources_and_session(
            &session,
            &root,
            (files.as_ref().clone(), Vec::new()),
            plan,
            None,
            sources.clone(),
        );
        assert!(facts
            .ts
            .get(&root.join("broken.ts"))
            .unwrap()
            .parse_error
            .is_some());
        super::super::outcomes::register_demand(
            &root,
            &config,
            consumer,
            &files,
            Some(&facts),
            &sources,
            &session,
        )
        .unwrap();
        let incomplete = session.check_failures();
        assert_eq!(
            !incomplete.is_empty(),
            failed,
            "{config_name}: {incomplete:?}"
        );
        if failed {
            assert!(
                incomplete.iter().all(|failure| failure.consumer == consumer
                    && failure.file.as_ref() == Some(&root.join("broken.ts"))),
                "{config_name}: {incomplete:?}"
            );
        }
    }
}

#[test]
fn demand_validation_failure_retains_independent_filesystem_findings() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check/filesystem-prerequisites"),
    );
    let snapshot = VisiblePathSnapshot::new(&root);
    let files = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    let config = crate::config::v2::load_v2_config_from_visible(
        &root,
        Some(&root.join("invalid-lint.yml")),
        &files,
    )
    .unwrap();
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    let facts = crate::codebase::check_facts::CheckFactMap::default();
    let findings = super::super::outcomes::run_filesystem_rules_with_config_snapshot_catalog_sources_facts_suppression_and_session(
        &root, &config, &files,
        super::super::PreparedFilesystemRuleInputs { session: Some(&session), next_config_resolver: None, snapshot: &snapshot, vitest_catalog: None, sources, workflow_documents: None, tsconfig_gate_project_inputs: None, config_path: None },
        Some(&facts), &session,
    ).unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding.rule == NO_EMPTY_OR_COMMENTS_ONLY_FILES
            && finding.file == "empty.txt"));
    assert!(session
        .check_failures()
        .iter()
        .any(|failure| failure.consumer == FORBIDDEN_LINT_SUPPRESSIONS
            && failure
                .message
                .contains("files must contain at least one non-empty glob")));
}

#[test]
fn invalid_source_demands_fail_instead_of_becoming_complete_checks() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check/filesystem-prerequisites"),
    );
    let candidates = [root.join("broken.ts")];
    let sources = crate::codebase::rules::source_store_for_files(&candidates);
    for (consumer, options, expected) in [
        (FINITE_SET_CONSISTENCY, "sets: false", "sets"),
        ("postgres-no-offset", "include: ['[']", "invalid glob"),
    ] {
        let config: NoMistakesConfig = serde_yaml::from_str(&format!(
            "rules:\n  - rule: {consumer}\n    scope: repository\n"
        ))
        .unwrap();
        let mut config = config;
        config.rules[0].options = serde_yaml::from_str(options).unwrap();
        let error = files(&root, &config, consumer, &candidates, None, &sources).unwrap_err();
        assert!(
            error.to_string().contains(expected),
            "{consumer}: {error:#}"
        );
    }
    assert_eq!(sources.physical_read_count(), 0);
}
