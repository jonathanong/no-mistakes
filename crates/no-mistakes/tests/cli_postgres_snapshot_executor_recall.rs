use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_executor_recall_check_json_snapshot() {
    let registry = "postgres-no-mutable-sql-registry";
    let lock = "postgres-lock-ordering";
    assert_saved_fixture(
        "postgres_executor_recall",
        &[
            (
                "registry_writer_alias_sink_direct",
                rules_path(registry, "fixture", "fail-reexport-writer-only"),
                rules_path(registry, "configs", "registry.yml"),
            ),
            (
                "registry_reexport_both_routes",
                rules_path(registry, "fixture", "fail-reexport-alias"),
                rules_path(registry, "configs", "registry.yml"),
            ),
            (
                "registry_named_writer_namespace_sink",
                rules_path(registry, "fixture", "alias-named-writer-namespace-sink"),
                rules_path(registry, "configs", "registry.yml"),
            ),
            (
                "registry_namespace_writer_named_sink",
                rules_path(registry, "fixture", "alias-namespace-writer-named-sink"),
                rules_path(registry, "configs", "registry.yml"),
            ),
            (
                "registry_require_writer_named_sink",
                rules_path(registry, "fixture", "alias-require-writer-named-sink"),
                rules_path(registry, "configs", "registry.yml"),
            ),
            (
                "registry_named_writer_require_sink",
                rules_path(registry, "fixture", "alias-named-writer-require-sink"),
                rules_path(registry, "configs", "registry.yml"),
            ),
            (
                "lock_whitespace_and_comments",
                rules_path(lock, "fixture", "fail-lock-trivia"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "source_string_marker_still_fails",
                rules_path(lock, "fixture", "fail-source-string-marker"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "sql_string_marker_still_fails",
                rules_path(lock, "fixture", "fail-sql-string-marker"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "long_source_string_marker_still_fails",
                rules_path(lock, "fixture", "fail-long-source-string"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "dollar_sql_string_marker_still_fails",
                rules_path(lock, "fixture", "fail-dollar-sql-string"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "tagged_dollar_sql_string_marker_still_fails",
                rules_path(lock, "fixture", "fail-tagged-dollar-sql-string"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "unicode_dollar_sql_string_marker_still_fails",
                rules_path(lock, "fixture", "fail-unicode-dollar-sql-string"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "ordinary_backslash_sql_string_marker_still_fails",
                rules_path(lock, "fixture", "fail-ordinary-backslash-sql-string"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "escape_string_marker_still_fails",
                rules_path(lock, "fixture", "fail-escape-string-marker"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "backslash_adjacent_comment_marker_exempts",
                rules_path(lock, "fixture", "pass-backslash-adjacent-comment"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "long_source_comment_marker_exempts",
                rules_path(lock, "fixture", "pass-long-source-comment"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "dollar_adjacent_comment_marker_exempts",
                rules_path(lock, "fixture", "pass-dollar-adjacent-comment"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "nested_sql_comment_marker_exempts",
                rules_path(lock, "fixture", "pass-nested-sql-comment"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "source_comment_marker_exempts",
                rules_path(lock, "fixture", "pass-source-comment-marker"),
                rules_path(lock, "configs", "default.yml"),
            ),
            (
                "sql_comment_marker_exempts",
                rules_path(lock, "fixture", "pass-sql-comment-marker"),
                rules_path(lock, "configs", "default.yml"),
            ),
        ],
    );
}
