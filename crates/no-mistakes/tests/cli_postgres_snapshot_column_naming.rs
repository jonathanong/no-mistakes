use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn repo_rel(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
}

#[test]
fn postgres_column_naming_check_json_snapshot() {
    let rule = "postgres-column-naming";
    // Output cases formerly asserted by the rule unit tests.
    let cases = [
        ("only_catalog_path", "only_catalog_path", "catalog-only.yml"),
        (
            "proposed_stale_allow",
            "proposed_stale_allow",
            "proposed.yml",
        ),
        (
            "allow_suppresses_real_finding",
            "allow_suppresses_real_finding",
            "proposed.yml",
        ),
        (
            "prepared_catalog_empty",
            "fixtures/postgres/column-naming/pass",
            "type-at.yml",
        ),
        (
            "custom_message",
            "fixtures/postgres/column-naming/pass",
            "type-created.yml",
        ),
        (
            "include_skips_catalog",
            "fixtures/postgres/column-naming/pass",
            "type-created-include.yml",
        ),
        (
            "exclude_skips_catalog",
            "fixtures/postgres/column-naming/pass",
            "type-created-exclude.yml",
        ),
        // due must not remain; expires must remain
        ("line_disable", "line-disable", "type-at.yml"),
        // due must not remain; expires must remain
        ("next_line_disable", "next-line-disable", "type-at.yml"),
        ("file_disable", "file-disable", "type-at.yml"),
        (
            "project_scoped_catalog",
            "fixtures/postgres/column-naming/project",
            "project-db.yml",
        ),
        (
            "numeric_negative_scale",
            "numeric_negative_scale",
            "numeric-amount.yml",
        ),
        (
            "typmod_title_name_passes",
            "typmod_title_name_passes",
            "typmod-cv.yml",
        ),
        ("typmod_title_fails", "typmod_title_fails", "typmod-cv.yml"),
        (
            "typmod_title_label_passes",
            "typmod_title_label_passes",
            "typmod-cv.yml",
        ),
        (
            "timestamp_typmod_passes",
            "timestamp_typmod_passes",
            "type-at.yml",
        ),
        (
            "weird_parenthetical_passes",
            "weird_parenthetical_passes",
            "keep-weird.yml",
        ),
        (
            "unclosed_typmod_passes",
            "unclosed_typmod_passes",
            "keep-weird.yml",
        ),
        (
            "label_text_fails_name_rule",
            "label_text_fails_name_rule",
            "typmod-cv.yml",
        ),
        ("types_due_timestamp", "types_due_timestamp", "types.yml"),
        ("types_active_boolean", "types_active_boolean", "types.yml"),
        ("types_since_date", "types_since_date", "types.yml"),
        (
            "types_shipped_at_two_findings",
            "types_shipped_at_two_findings",
            "types.yml",
        ),
        (
            "types_paid_at_without_time_zone",
            "types_paid_at_without_time_zone",
            "types.yml",
        ),
        ("types_cursor_text", "types_cursor_text", "types.yml"),
        ("valid_type_names", "valid_type_names", "types.yml"),
        (
            "boolean_type_keeps_stored_case",
            "boolean_type_keeps_stored_case",
            "bool-is.yml",
        ),
        (
            "missing_hint_omitted",
            "missing_hint_omitted",
            "type-at.yml",
        ),
        (
            "name_rule_missing_hint",
            "name_rule_missing_hint",
            "name-at.yml",
        ),
        (
            "every_type_rule_reports",
            "every_type_rule_reports",
            "two-bool-hints.yml",
        ),
        (
            "every_name_rule_reports",
            "every_name_rule_reports",
            "two-name-hints.yml",
        ),
        (
            "skip_generated_columns",
            "skip_generated_columns",
            "skip-generated-true.yml",
        ),
        (
            "check_generated_columns",
            "skip_generated_columns",
            "skip-generated-false.yml",
        ),
        (
            "allow_suppresses_column",
            "allow_suppresses_column",
            "allow-shipped.yml",
        ),
        (
            "forbidden_target_table_text",
            "forbidden_target_table_text",
            "forbidden.yml",
        ),
        (
            "forbidden_table_name",
            "forbidden_table_name",
            "forbidden.yml",
        ),
        (
            "forbidden_target_table_integer",
            "forbidden_target_table_integer",
            "forbidden.yml",
        ),
        (
            "forbidden_timetable_passes",
            "forbidden_timetable_passes",
            "forbidden.yml",
        ),
        (
            "forbidden_first_pattern_only",
            "forbidden_first_pattern_only",
            "forbidden-first.yml",
        ),
        ("suffix_owner_id", "suffix_owner_id", "suffix.yml"),
        ("suffix_owner_user_id", "suffix_owner_user_id", "suffix.yml"),
        ("suffix_created_by_id", "suffix_created_by_id", "suffix.yml"),
        ("suffix_revoked_by_id", "suffix_revoked_by_id", "suffix.yml"),
        ("suffix_exact_user_id", "suffix_exact_user_id", "suffix.yml"),
        ("target_off_item", "target_off_item", "target-off.yml"),
        ("target_off_batch", "target_off_batch", "target-off.yml"),
        ("last_word_item", "last_word_item", "last-word.yml"),
        (
            "last_word_batch_passes",
            "last_word_batch_passes",
            "last-word.yml",
        ),
        ("full_name_batch", "full_name_batch", "full-name.yml"),
        (
            "full_name_import_batch_id",
            "full_name_import_batch_id",
            "full-name.yml",
        ),
        ("full_name_item", "full_name_item", "full-name.yml"),
        ("suggest_currency", "suggest_currency", "last-word.yml"),
        (
            "currency_code_passes",
            "currency_code_passes",
            "last-word.yml",
        ),
        (
            "suggest_order_currency",
            "suggest_order_currency",
            "last-word.yml",
        ),
        (
            "archived_snapshot_id",
            "archived_snapshot_id",
            "last-word.yml",
        ),
        (
            "archived_order_id_passes",
            "archived_order_id_passes",
            "last-word.yml",
        ),
        ("article_id_passes", "article_id_passes", "last-word.yml"),
        (
            "order_line_item_passes",
            "order_line_item_passes",
            "last-word.yml",
        ),
        ("singular_person", "singular_person", "last-word.yml"),
        ("singular_status", "singular_status", "last-word.yml"),
        ("singular_staff", "singular_staff", "last-word.yml"),
        (
            "descriptive_github_user_id",
            "descriptive_github_user_id",
            "last-word.yml",
        ),
        (
            "github_account_id_passes",
            "github_account_id_passes",
            "last-word.yml",
        ),
        (
            "full_name_natural_import_batch",
            "full_name_natural_import_batch",
            "full-name.yml",
        ),
        (
            "target_off_github_user_id",
            "target_off_github_user_id",
            "target-off.yml",
        ),
        (
            "external_id_last_word",
            "external_id_last_word",
            "last-word.yml",
        ),
        (
            "account_github_user_id_passes",
            "account_github_user_id_passes",
            "last-word.yml",
        ),
        (
            "primary_key_id_passes",
            "primary_key_id_passes",
            "last-word.yml",
        ),
        (
            "self_reference_off_passes",
            "self_reference_off_passes",
            "last-word.yml",
        ),
        (
            "self_reference_checked",
            "self_reference_checked",
            "self-ref.yml",
        ),
        (
            "foreign_zero_referenced_columns",
            "foreign_zero_referenced_columns",
            "last-word.yml",
        ),
        (
            "qualified_composite_natural_key",
            "qualified_composite_natural_key",
            "qualified-composite.yml",
        ),
        (
            "repeated_natural_key",
            "repeated_natural_key",
            "last-word.yml",
        ),
        (
            "target_name_first_pattern",
            "target_name_first_pattern",
            "target-early.yml",
        ),
        (
            "target_name_unmatched_orders",
            "target_name_unmatched_orders",
            "target-missing-groups.yml",
        ),
        (
            "target_name_literal_dollar",
            "target_name_literal_dollar",
            "target-missing-groups.yml",
        ),
        (
            "leading_underscore_id",
            "leading_underscore_id",
            "last-word.yml",
        ),
        (
            "quoted_users_target",
            "quoted_users_target",
            "quoted-users.yml",
        ),
        (
            "quoted_schema_users_target",
            "quoted_schema_users_target",
            "quoted-other-users.yml",
        ),
        (
            "quoted_lowercase_users",
            "quoted_lowercase_users",
            "quoted-users.yml",
        ),
        (
            "quoted_suffix_users",
            "quoted_suffix_users",
            "suffix-quoted.yml",
        ),
        (
            "quoted_suffix_schema_users",
            "quoted_suffix_schema_users",
            "suffix-quoted.yml",
        ),
        (
            "quoted_suffix_created_by",
            "quoted_suffix_created_by",
            "suffix-quoted.yml",
        ),
        (
            "quoted_suffix_schema_created_by",
            "quoted_suffix_schema_created_by",
            "suffix-quoted.yml",
        ),
        // lower-case users is not the listed Users table
        (
            "quoted_suffix_lowercase_not_listed",
            "quoted_suffix_lowercase_not_listed",
            "suffix-quoted.yml",
        ),
    ]
    .map(|(label, root, config)| {
        let fixture = if root.starts_with("fixtures/") {
            repo_rel(root)
        } else {
            rules_path(rule, "fixture", root)
        };
        (label, fixture, rules_path(rule, "configs", config))
    });
    assert_saved_fixture("postgres_column_naming", &cases);
}
