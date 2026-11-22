use super::*;
use crate::codebase::check_outcome::Prerequisite;
use std::path::PathBuf;
#[test]
fn exhausted_value_flow_preserves_previous_graph_prerequisite_failures() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/foreign-receiver-work-budget/fixture"),
    );
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let file = root.join("src/shared-2048.mts");
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        vec![file.clone()],
        crate::codebase::check_facts::CheckFactPlan {
            graph: crate::codebase::ts_source::facts::TsFactPlan {
                imports: true,
                value_flow: true,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let session = AnalysisSession::disabled();
    let previous = CheckFailure::new("configured-sibling", "existing prerequisite failed");
    session.set_check_prerequisite(
        "codebase.canonical-graph",
        Prerequisite::Failed(vec![previous.clone()]),
    );
    register_value_flow_failures(&config, &shared, &session);
    let Some(Prerequisite::Failed(failures)) =
        session.check_prerequisite("codebase.canonical-graph")
    else {
        panic!("exhausted graph cannot become ready");
    };
    assert!(failures.contains(&previous));
    assert!(failures
        .iter()
        .any(|failure| failure.consumer == UNCONSTRUCTED_ERROR_CLASS
            && failure.file.as_ref() == Some(&file)));
}

#[test]
fn fragment_coverage_is_scoped_by_the_rule_and_global_graph_errors_still_propagate() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases");
    let root = crate::codebase::ts_resolver::normalize_path(
        &fixtures.join("codebase-analysis/foreign-receiver-work-budget/fixture"),
    );
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let fragment_configs = fixtures.join("rules/postgres-sql-fragment-escape/configs");
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        vec![root.join("src/shared-2048.mts")],
        crate::codebase::check_facts::CheckFactPlan {
            graph: crate::codebase::ts_source::facts::TsFactPlan {
                imports: true,
                value_flow: true,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    for (file, demands_analysis) in [("empty-tags.yml", false), ("append-chain.yml", true)] {
        let fragment_config =
            crate::config::v2::load_v2_config(&root, Some(&fragment_configs.join(file))).unwrap();
        let mut config = config.clone();
        config.rules.extend(fragment_config.rules);

        let session = AnalysisSession::disabled();
        register_value_flow_failures(&config, &shared, &session);
        assert!(session.check_consumer_ready(postgres_sql_fragment_escape::RULE_ID));
        assert!(!session.check_consumer_ready(UNCONSTRUCTED_ERROR_CLASS));
        assert!(matches!(
            session.check_prerequisite("codebase.canonical-graph"),
            Some(Prerequisite::Failed(_))
        ));

        let session = AnalysisSession::disabled();
        graph_failed(
            &config,
            &session,
            &crate::codebase::check_outcome::CheckInputFailure::source(
                root.join("src/shared-2048.mts"),
                "missing graph facts".to_owned(),
            )
            .into(),
        );
        assert!(session.check_consumer_ready(postgres_sql_fragment_escape::RULE_ID));
        assert!(!session.check_consumer_ready(UNCONSTRUCTED_ERROR_CLASS));

        let session = AnalysisSession::disabled();
        graph_failed(&config, &session, &anyhow::anyhow!("graph failed"));
        assert_eq!(
            session.check_consumer_ready(postgres_sql_fragment_escape::RULE_ID),
            !demands_analysis
        );
        assert!(!session.check_consumer_ready(UNCONSTRUCTED_ERROR_CLASS));
    }
}
