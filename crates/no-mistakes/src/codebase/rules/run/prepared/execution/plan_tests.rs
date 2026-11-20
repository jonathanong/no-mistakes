use super::{ready_graph_plan, Prerequisite};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_outcome::CheckFailure;
use crate::codebase::rules::UNCONSTRUCTED_ERROR_CLASS;
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;

fn fragment_config() -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "postgres-sql-fragment-escape".to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(
            "trustedSqlTags:\n  - {module: \"@example/sql\", name: default}",
        )
        .expect("options"),
        ..RuleDef::default()
    });
    config
}

#[test]
fn incomplete_value_flow_keeps_a_graph_for_fragment_escape() {
    let config = fragment_config();
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    session.set_check_prerequisite(
        "codebase.canonical-graph",
        Prerequisite::Failed(vec![CheckFailure {
            consumer: UNCONSTRUCTED_ERROR_CLASS.to_string(),
            file: None,
            property: None,
            message: "partial".to_string(),
        }]),
    );
    let restored = ready_graph_plan(&config, &session, Prerequisite::Failed(Vec::new()));
    assert!(matches!(restored, Prerequisite::Ready(Some(_))));
}

#[test]
fn a_failed_fragment_escape_consumer_does_not_restore_the_graph() {
    let config = fragment_config();
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    session.record_check_failure(CheckFailure {
        consumer: "postgres-sql-fragment-escape".to_string(),
        file: None,
        property: None,
        message: "graph input failed".to_string(),
    });
    let restored = ready_graph_plan(&config, &session, Prerequisite::Failed(Vec::new()));
    assert!(matches!(restored, Prerequisite::Failed(_)));
}

#[test]
fn partial_checks_are_required_to_keep_the_fragment_escape_graph() {
    let config = fragment_config();
    let session = AnalysisSession::disabled();
    let restored = ready_graph_plan(&config, &session, Prerequisite::Failed(Vec::new()));
    assert!(matches!(restored, Prerequisite::Failed(_)));
}
