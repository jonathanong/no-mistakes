use super::tests::{config_with_options, fixture};
use super::*;

// Output cases moved to cli_postgres_snapshot_no_offset.

#[test]
fn unknown_unanalyzable_mode_is_a_config_error() {
    let root = fixture("unanalyzable");
    let error = check_with_files(
        &root,
        &config_with_options("unanalyzableSql: sometimes"),
        &[root.join("src/query.ts")],
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("postgres-no-offset: unanalyzableSql must be `fail`, `ignore`, or `advise`"),
        "{error}"
    );
}
