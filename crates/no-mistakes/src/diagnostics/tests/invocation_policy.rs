#[test]
fn cli_and_napi_entrypoints_select_distinct_parent_signal_policies() {
    let cli = include_str!("../../main/invocation.rs");
    let tasks = include_str!("../../napi_api/async_task.rs");
    let napi = include_str!("../../napi_api/async_task/observed.rs");

    assert!(cli.contains("ExecutionGuard::acquire_for_cli"));
    assert_eq!(tasks.matches("observed::execute_json(").count(), 2);
    assert!(napi.contains("InvocationGuard::acquire(invocation_options)"));
    assert!(!napi.contains("InvocationGuard::acquire_for_cli"));
}

#[test]
fn production_test_planning_subprocesses_use_deadline_aware_output() {
    let sources = [
        include_str!("../../tests/changed_files.rs"),
        include_str!("../../tests/diff_parser.rs"),
        include_str!("../../tests/lockfile_changes.rs"),
    ];

    assert!(sources.iter().all(|source| !source.contains(".output()")));
}
