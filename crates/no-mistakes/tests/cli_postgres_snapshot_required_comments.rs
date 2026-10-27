use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn catalog() -> PathBuf {
    rules_path(
        "postgres-required-comments",
        "../../../fixtures/postgres",
        "required-comments",
    )
}

#[test]
fn postgres_required_comments_check_json_snapshot() {
    let rule = "postgres-required-comments";
    let root = catalog();
    let config = |name: &str| rules_path(rule, "configs", name);
    // repeated_runs_are_identical is the jobs 1 vs 2 check on all_objects.
    // finding_text_helpers_cover_each_listed_sentence is the same case: these
    // sentences remain, each containing " Add COMMENT ON ":
    // table has no COMMENT ON TABLE; table comment is shorter than 10
    // characters; column has no COMMENT ON COLUMN; view has no COMMENT ON VIEW;
    // materialized view has no COMMENT ON MATERIALIZED VIEW.
    assert_saved_fixture(
        "postgres_required_comments",
        &[
            // Must contain table:events, table:blank_rows, and
            // table:schema_migrations ("table has no COMMENT ON TABLE").
            // Must not contain table:orders:, table:short_rows:, column:,
            // view:, or materialized-view:.
            ("default_tables", root.clone(), config("default-tables.yml")),
            // short_rows is shorter than 10; blank_rows has no COMMENT.
            // table:orders: is absent.
            ("min_length_10", root.clone(), config("min-length-10.yml")),
            // Must contain events, short_rows, orders.secret, orders.code,
            // orders.title, open_orders, blank_view, tiny_view, and paid_orders.
            // Absent: table:orders:, column:orders.note, column:orders.id,
            // column:orders.vendor_id, column:events.payload, view:titled_orders.
            ("all_objects", root.clone(), config("all-objects.yml")),
            // Contains secret, code, and title. Absent: note, vendor_id, id,
            // events.payload, schema_migrations.version.
            (
                "column_name_patterns",
                root.clone(),
                config("column-patterns.yml"),
            ),
            // Contains secret, title, and schema_migrations.version.
            // Absent: note, code, id, vendor_id, events.payload, blank_rows.id.
            (
                "empty_column_patterns",
                root.clone(),
                config("empty-column-patterns.yml"),
            ),
            // ^payload$ reports nothing: the partitioned parent comment counts.
            // table:events: remains on default_tables.
            (
                "payload_columns",
                root.clone(),
                config("payload-columns.yml"),
            ),
            // open_orders and blank_view remain. tiny_view and
            // materialized-view are absent.
            ("views", root.clone(), config("views.yml")),
            // paid_orders remains. view:open_orders, view:blank_view, and
            // view:tiny_view are absent.
            (
                "materialized_views",
                root.clone(),
                config("materialized-views.yml"),
            ),
            // schema_migrations, orders.secret, open_orders, and paid_orders
            // are absent. Stale entry table:not_real remains, and table:events
            // remains.
            ("allow_stale", root.clone(), config("allow.yml")),
            ("include_misses_catalog", root, config("include-misses.yml")),
        ],
    );
}
