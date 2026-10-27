use super::*;

// Output cases moved to cli_postgres_snapshot_generated_column_writes.

#[test]
fn unknown_unanalyzable_mode_is_a_config_error() {
    let root = unit_fixture("unanalyzable");
    let error = check_with_files(
        &root,
        &config_with_options("unanalyzableSql: sometimes"),
        &[root.join("write.ts")],
    )
    .err()
    .unwrap()
    .to_string();
    assert!(
        error.contains(
            "postgres-no-generated-column-writes: unanalyzableSql must be `fail`, `ignore`, or `advise`"
        ),
        "{error}"
    );
}
