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
