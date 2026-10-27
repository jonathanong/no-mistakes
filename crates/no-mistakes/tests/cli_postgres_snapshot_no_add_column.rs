use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_no_add_column_check_json_snapshot() {
    let rule = "postgres-no-add-column";
    let migration = rules_path(rule, "configs", "migration-001.yml");
    let allowed = rules_path(rule, "configs", "allowed-draft.yml");
    // sqlInclude is the single file each former check_with_files call passed.
    // duplicate_allowed_migrations and
    // duplicate_allowed_migration_does_not_also_report_as_stale share one case:
    // the duplicate entry is not also reported as stale.
    assert_saved_fixture(
        "postgres_no_add_column",
        &[
            (
                "alter_table_add_column",
                rules_path(rule, "fixture", "fail"),
                migration.clone(),
            ),
            // Must not contain "ALTER TABLE ADD COLUMN".
            (
                "pass_postgres_regress",
                rules_path(rule, "fixture", "pass-postgres-regress"),
                migration.clone(),
            ),
            // One finding, on line 3.
            (
                "fail_comment_before_add",
                rules_path(rule, "fixture", "fail-comment-before-add"),
                migration.clone(),
            ),
            (
                "add_column_inside_do_block",
                rules_path(rule, "fixture", "fail-do-block"),
                migration.clone(),
            ),
            (
                "create_table_and_add_constraint",
                rules_path(rule, "fixture", "pass"),
                migration,
            ),
            (
                "allowed_migration",
                rules_path(rule, "fixture", "allowed"),
                allowed.clone(),
            ),
            (
                "allowed_migration_do_block",
                rules_path(rule, "fixture", "allowed-do"),
                allowed,
            ),
            // Both remain: posts.status does not match an allowedMigrations entry,
            // and the stale target
            // migrations/001.sql:posts:status:TEXT:false:'published'.
            (
                "unexpected_and_stale_allowed_migrations",
                rules_path(rule, "fixture", "mismatch"),
                rules_path(rule, "configs", "allowed-published.yml"),
            ),
            // dynamic_identifier.dynamic_identifier does not match, and a stale
            // allowedMigrations entry remains.
            (
                "static_format_shape_and_stale_allowlist",
                rules_path(rule, "fixture", "dynamic"),
                rules_path(rule, "configs", "allowed-draft.yml"),
            ),
            (
                "duplicate_allowed_migrations",
                rules_path(rule, "fixture", "allowed"),
                rules_path(rule, "configs", "duplicate-draft.yml"),
            ),
            // CLI sees the suppressed file only. The pre-suppression count stays
            // in honors_disable_comments.
            (
                "disabled_next_line",
                rules_path(rule, "fixture", "fail"),
                rules_path(rule, "configs", "disabled.yml"),
            ),
        ],
    );
}
