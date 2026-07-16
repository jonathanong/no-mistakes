use super::*;
use std::time::Duration;

#[test]
fn flow_query_returns_timeout_instead_of_a_partial_report() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/simple/fixture"),
    );
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root.clone(),
        Default::default(),
        Default::default(),
    );
    let options = FlowOptions {
        target: "entry.ts".to_string(),
        root: root.clone(),
        tsconfig: None,
        config: None,
        direction: FlowDirection::Deps,
        depth: 1,
        relationships: vec![RelationshipArg::Import],
    };
    let _guard = crate::invocation::install_test_deadline(Duration::ZERO).unwrap();

    let error = run_with_prepared_graph(&options, &root, &graph).unwrap_err();

    assert_eq!(crate::invocation::timeout_exit_code(&error), Some(124));
}

#[test]
fn flow_traversal_checks_periodic_deadlines() {
    assert!(check_traversal_timeout(256).is_ok());
}
