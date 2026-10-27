use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_idempotent_insert_check_json_snapshot() {
    let rule = "postgres-idempotent-insert";
    // Finding counts, messages, lines, emptiness, and option effects from
    // check_with_files. A copied disabled.sql without the directive is the
    // unsuppressed side.
    let cases = [
        ("insert_without_conflict", "insert-only", "sql.yml"),
        ("do_nothing", "pass-do-nothing", "sql.yml"),
        // One assigned EXCLUDED column proves the WHERE skip.
        ("partial_where", "pass-partial-where", "sql.yml"),
        ("volatile_where_proof", "fail-volatile-proof", "sql.yml"),
        ("not_exists", "pass-not-exists", "sql.yml"),
        ("volatile_update", "fail-volatile", "sql.yml"),
        ("disabled_insert", "disabled-raw", "sql.yml"),
        ("disabled_insert_suppressed", "disabled", "sql.yml"),
        ("psql_insert", "fail", "psql.yml"),
        ("embedded_skipped", "skip-embedded", "embedded-off.yml"),
        ("unparseable_ignored", "unparseable", "ignore.yml"),
        ("unparseable_flagged", "unparseable", "sql.yml"),
        ("exclude_insert", "insert-only", "exclude-insert.yml"),
        ("include_insert", "insert-only", "include-insert.yml"),
        ("include_missing", "insert-only", "include-missing.yml"),
        ("dynamic_insert", "fail-dynamic", "embedded.yml"),
        (
            "dynamic_insert_ignored",
            "fail-dynamic",
            "dynamic-ignore.yml",
        ),
        ("embedded_insert", "fail-embedded", "embedded.yml"),
        (
            "embedded_scan_disabled",
            "fail-embedded",
            "embedded-off.yml",
        ),
        (
            "later_trigger_dropped",
            "pass-trigger-dropped-later",
            "sql.yml",
        ),
        (
            "live_before_insert_trigger",
            "fail-live-before-insert-trigger",
            "sql.yml",
        ),
        (
            "trigger_dropped_before_insert",
            "pass-trigger-dropped-before-insert",
            "sql.yml",
        ),
        (
            "trigger_dropped_after_insert",
            "fail-trigger-dropped-after-insert",
            "sql.yml",
        ),
        (
            "trigger_replaced_before_insert",
            "pass-trigger-replaced-before-insert",
            "sql.yml",
        ),
        (
            "trigger_replaced_after_insert",
            "fail-trigger-replaced-after-insert",
            "sql.yml",
        ),
        (
            "embedded_trigger_live",
            "embedded-trigger-final-live",
            "embedded-trigger.yml",
        ),
        (
            "embedded_trigger_dropped",
            "embedded-trigger-final-dropped",
            "embedded-trigger.yml",
        ),
        (
            "embedded_trigger_replaced",
            "embedded-trigger-final-replaced",
            "embedded-trigger.yml",
        ),
        (
            "drop_keywords_in_comments",
            "pass-trigger-drop-keywords-in-comments",
            "sql.yml",
        ),
        (
            "dropped_trigger_keeps_noop_proof",
            "pass-trigger-dropped-rewriter",
            "rewriter.yml",
        ),
        (
            "same_line_create_insert_drop",
            "fail-trigger-same-line-create-insert-drop",
            "sql.yml",
        ),
        (
            "same_line_drop_insert_create",
            "pass-trigger-same-line-drop-insert-create",
            "sql.yml",
        ),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(rule, "fixture", fixture),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_idempotent_insert", &cases);
}
