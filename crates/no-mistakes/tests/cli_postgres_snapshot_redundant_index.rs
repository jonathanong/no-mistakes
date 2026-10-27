use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_redundant_index_check_json_snapshot() {
    let rule = "postgres-redundant-index";
    let fixture = |name: &str| rules_path(rule, "fixture", name);
    let config = |name: &str| rules_path(rule, "configs", name);
    let one = config("migrations-001.yml");
    let pair = config("migrations-0001-0002.yml");
    // sqlInclude lists only the migration files present in that fixture, which
    // is the file list the former run() helper passed.
    assert_saved_fixture(
        "postgres_redundant_index",
        &[
            // One finding, target events.idx_events__topic_id, message names
            // idx_events__topic_id__created_at.
            ("strict_prefix_btree", fixture("fail"), one.clone()),
            ("unique_shorter_index", fixture("pass"), one.clone()),
            ("same_line_directive", fixture("allow"), one.clone()),
            ("later_drop", fixture("dropped"), pair.clone()),
            ("concurrent_drop", fixture("concurrent-drop"), pair.clone()),
            (
                "allowed_index",
                fixture("fail"),
                config("allowed-topic.yml"),
            ),
            // Some message contains "stale".
            (
                "stale_allowed_index",
                fixture("pass"),
                config("stale-never.yml"),
            ),
            (
                "custom_allow_directive",
                fixture("allow"),
                config("custom-directive.yml"),
            ),
            ("schema_qualified_tables", fixture("schema"), one.clone()),
            // One finding, target public.events.idx_events__topic_id.
            (
                "drop_index_other_schema",
                fixture("schema-drop"),
                config("migrations-001-0002.yml"),
            ),
            ("drop_table", fixture("dropped-table"), pair.clone()),
            (
                "numeric_later_drop",
                fixture("numeric-order"),
                config("migrations-2-10.yml"),
            ),
            (
                "flyway_version_prefix_drop",
                fixture("flyway-order"),
                config("migrations-flyway.yml"),
            ),
            ("multiline_create_index", fixture("multiline"), one),
            (
                "earlier_drop_keeps_later_create",
                fixture("earlier-drop"),
                pair,
            ),
            (
                "predicate_string_literals",
                fixture("predicate-case"),
                config("migrations-001.yml"),
            ),
        ],
    );
}
