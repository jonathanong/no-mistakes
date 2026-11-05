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
