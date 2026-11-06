use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, GraphFiles};
use crate::codebase::ts_resolver::TsConfig;
use crate::codebase::ts_source::facts::TsFactPlan;
use std::path::PathBuf;

#[test]
fn production_consumer_rejects_exhausted_shared_facts_without_blocking_extends_graphs() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/foreign-receiver-work-budget/fixture"),
    );
    let names = [
        "argument-diamond-16.mts",
        "argument-diamond-20.mts",
        "shared-2048.mts",
        "shared-4096.mts",
    ];
    let files: Vec<_> = names
        .iter()
        .map(|name| root.join("src").join(name))
        .collect();
    let facts = collect_check_facts(
        &root,
        files.clone(),
        CheckFactPlan {
            graph: TsFactPlan {
                imports: true,
                function_calls: true,
                value_flow: true,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let graph = DepGraph::build_with_plan_files_config_and_facts(
        &root,
        &TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..Default::default()
        },
        GraphBuildPlan {
            extends: true,
            ..Default::default()
        },
        &GraphFiles::from_files(files.clone()),
        None,
        Some(&facts),
    )
    .expect("extends-only queries retain their complete syntactic facts");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("production-config.yml")))
            .unwrap();
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    let diagnostics =
        super::check_with_graph_and_session(&root, &config, &graph, &facts, Some(&session))
            .unwrap();
    assert!(diagnostics.findings.is_empty());
    assert!(diagnostics.advisories.is_empty());
    let failures = session.check_failures();
    assert_eq!(failures.len(), names.len());
    let mut failed_files: Vec<_> = failures
        .iter()
        .map(|failure| {
            assert_eq!(failure.consumer, super::RULE_ID);
            assert!(failure.message.contains("per-file work budget"));
            failure.file.clone().unwrap()
        })
        .collect();
    failed_files.sort();
    assert_eq!(failed_files, files);
    for session in [None, Some(AnalysisSession::disabled())] {
        let error =
            super::check_with_graph_and_session(&root, &config, &graph, &facts, session.as_deref())
                .err()
                .expect("nonpartial consumers must reject exhausted facts");
        let failure = error
            .downcast_ref::<crate::codebase::check_outcome::CheckInputFailure>()
            .expect("failure retains its source identity");
        assert!(failure.message.contains("per-file work budget"));
        assert!(files.contains(failure.file.as_ref().unwrap()));
    }
}
