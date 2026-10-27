use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_column_naming_more_check_json_snapshot() {
    let rule = "postgres-column-naming";
    // Output cases formerly asserted by the rule unit tests.
    let cases = [
        (
            "reserved_reviewer_user_id",
            "reserved_reviewer_user_id",
            "reserved.yml",
        ),
        ("reserved_user_id", "reserved_user_id", "reserved.yml"),
        (
            "reserved_github_user_id_text_passes",
            "reserved_github_user_id_text_passes",
            "reserved.yml",
        ),
        ("reserved_page_url", "reserved_page_url", "reserved.yml"),
        ("reserved_links_url", "reserved_links_url", "reserved.yml"),
        (
            "reserved_page_url_id_passes",
            "reserved_page_url_id_passes",
            "reserved.yml",
        ),
        ("reserved_any_type", "reserved_any_type", "reserved-any.yml"),
        (
            "reserved_wrong_target",
            "reserved_wrong_target",
            "reserved.yml",
        ),
        (
            "assignee_reserved_and_target",
            "assignee_reserved_and_target",
            "assignee.yml",
        ),
        (
            "composite_without_follow",
            "composite_without_follow",
            "reserved.yml",
        ),
        (
            "composite_leading_targets",
            "composite_leading_targets",
            "follow-true.yml",
        ),
        (
            "composite_follow_dead_end",
            "composite_follow_dead_end",
            "follow-true.yml",
        ),
        ("follow_two_hops", "follow_two_hops", "follow-true.yml"),
        ("follow_renames", "follow_renames", "follow-true.yml"),
        (
            "follow_either_target",
            "follow_either_target",
            "follow-true.yml",
        ),
        ("follow_cycle", "follow_cycle", "follow-true.yml"),
        ("require_order_id", "require_order_id", "require.yml"),
        (
            "require_without_rule_passes",
            "require_without_rule_passes",
            "catalog-only.yml",
        ),
        (
            "require_exempt_matches",
            "require_exempt_matches",
            "exempt.yml",
        ),
        (
            "require_uuid_array_passes",
            "require_uuid_array_passes",
            "require.yml",
        ),
        (
            "require_generated_passes",
            "require_generated_passes",
            "require.yml",
        ),
        (
            "require_foreign_key_passes",
            "require_foreign_key_passes",
            "require.yml",
        ),
        (
            "require_position_passes",
            "require_position_passes",
            "require.yml",
        ),
        (
            "require_reserved_column_passes",
            "require_reserved_column_passes",
            "require.yml",
        ),
        (
            "stale_cursor_exempt",
            "stale_cursor_exempt",
            "cursor-exempt.yml",
        ),
        (
            "stale_cursor_exempt_fixture",
            "stale_cursor_exempt_fixture",
            "cursor-exempt.yml",
        ),
        (
            "stale_never_exempt",
            "stale_never_exempt",
            "never-exempt.yml",
        ),
        // the one finding must say this uuid column has no foreign key
        (
            "reserved_blocks_require",
            "reserved_blocks_require",
            "reserved-and-require.yml",
        ),
        (
            "ignored_table_stale_exempt",
            "ignored_table_stale_exempt",
            "ignore-link.yml",
        ),
        ("bare_suffix_pass", "bare-suffix-pass", "bare-suffix.yml"),
        // orders.owner_id and audits.xuser_id must remain; must not contain user_user_id
        ("bare_suffix_fail", "bare-suffix-fail", "bare-suffix.yml"),
        // orders.owner_id and audits.xuser_id must remain; must not contain user_user_id
        (
            "bare_suffix_fail_allow_referenced",
            "bare-suffix-fail",
            "bare-suffix-allow-ref.yml",
        ),
        (
            "natural_key_pass",
            "natural-key-pass",
            "natural-allow-last.yml",
        ),
        (
            "natural_key_pass_full_name",
            "natural-key-pass",
            "natural-allow-full.yml",
        ),
        (
            "natural_key_default_pass",
            "natural-key-default-pass",
            "natural-full.yml",
        ),
        (
            "natural_key_default_fail",
            "natural-key-default-fail",
            "natural-last.yml",
        ),
        (
            "natural_key_default_fail_explicit_off",
            "natural-key-default-fail",
            "natural-last-off.yml",
        ),
        // currency, item_id, and xdelivery_key must remain
        (
            "natural_key_fail",
            "natural-key-fail",
            "natural-allow-last.yml",
        ),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(rule, "fixture", fixture),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_column_naming_more", &cases);
}
