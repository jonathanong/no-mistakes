use super::*;
use no_mistakes::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use no_mistakes::codebase::ts_source::VisiblePathSnapshot;

#[test]
fn task_without_session_keeps_legacy_prepared_react_findings() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/react-traits-config/assert-no-fetch/fixture");
    let snapshot = VisiblePathSnapshot::new(&root);
    let facts = collect_check_facts(
        &root,
        snapshot.paths_for(&root).to_vec(),
        CheckFactPlan {
            react: true,
            ..Default::default()
        },
    );
    let config = no_mistakes::config::v2::load_v2_config(&root, None).unwrap();
    let prepared = react_traits::prepare_check_from_loaded_config(&config, false);
    let expected =
        react_traits::run_check_with_prepared_facts_for_aggregate(&root, &[], &facts, &prepared)
            .unwrap();
    assert!(!expected.findings.is_empty());
    let task = run_react_check_with_session(
        &root,
        true,
        &facts,
        &prepared,
        &snapshot.source_store_for(&root),
        true,
        None,
    )
    .unwrap();
    assert!(task.warning.is_none());
    assert_eq!(
        serde_json::to_value(task.findings).unwrap(),
        serde_json::to_value(expected.findings).unwrap()
    );
    assert!(!task.react_suppression_targets.is_empty());
}
