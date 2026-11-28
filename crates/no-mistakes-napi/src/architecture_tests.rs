#[test]
fn analyze_project_check_prepare_reuses_a_supplied_test_filter() {
    let check_prepare = concat!(
        include_str!("analyze_project/context/check_prepare.rs"),
        include_str!("analyze_project/context/check_prepare_graph.rs"),
    );

    assert!(check_prepare.contains("graph_plan::prepare("));
    assert!(check_prepare.contains("codebase_config: &prepared.codebase_config"));
    assert!(check_prepare.contains("visible_paths: prepared.visible_paths.as_ref()"));
    assert!(!check_prepare.contains("prepare_graph_config("));
    assert!(!check_prepare.contains("TestFileFilter::new("));
}

#[test]
fn napi_entrypoints_select_non_cli_parent_signal_policy() {
    let tasks = include_str!("async_task.rs");
    let napi = include_str!("async_task/observed.rs");

    assert_eq!(tasks.matches("observed::execute_json(").count(), 2);
    assert!(napi.contains("InvocationGuard::acquire(invocation_options)"));
    assert!(!napi.contains("InvocationGuard::acquire_for_cli"));
}
