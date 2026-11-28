#[test]
fn cli_entrypoint_selects_cli_parent_signal_policy() {
    let cli = include_str!("../../main/invocation.rs");

    assert!(cli.contains("ExecutionGuard::acquire_for_cli"));
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
