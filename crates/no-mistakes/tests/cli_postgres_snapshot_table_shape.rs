use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn catalog() -> PathBuf {
    rules_path(
        "postgres-table-shape",
        "../../../fixtures/postgres",
        "table-shape",
    )
}

#[test]
fn postgres_table_shape_check_json_snapshot() {
    let rule = "postgres-table-shape";
    let root = catalog();
    let config = |name: &str| rules_path(rule, "configs", name);
    // valid_examples_and_edges_pass and reports_every_specified_finding_text
    // share main. empty_shapes_report_nothing_and_runs_match's equality is the
    // jobs 1 vs 2 check on main; the empty catalog is empty_shapes.
    // Absent tables: order_revisions, orders, article_votes, thread_votes,
    // order_status_changes, split_changes, qualified_changes, email_work_items,
    // domain_events, report_export_cursors, search_reindex_cursors,
    // billing_reconciliation_cursors, only_revisions, vendor_sync_history,
    // nullable_revisions, ref_pattern. Absent phrases: "primary key column job"
    // and "table:draft_changes: foreign key on changed_by_id must reference".
    // Required texts from reports_every_specified_finding_text stay in main.
    assert_saved_fixture(
        "postgres_table_shape",
        &[
            ("main", root.clone(), config("main.yml")),
            // alpha and beta missing columns, plus banned pattern ^orders$.
            (
                "edge_two_shapes",
                root.clone(),
                config("edge-two-shapes.yml"),
            ),
            ("edge_unchecked", root.clone(), config("edge-unchecked.yml")),
            (
                "edge_ref_pattern",
                root.clone(),
                config("edge-ref-pattern.yml"),
            ),
            // FOR EACH STATEMENT trigger executing fn_reject_mutation().
            (
                "edge_statement_trigger",
                root.clone(),
                config("edge-statement-trigger.yml"),
            ),
            // column revised_by_id must be nullable.
            ("edge_nullable", root.clone(), config("edge-nullable.yml")),
            ("empty_shapes", root.clone(), config("empty-shapes.yml")),
            // table:order_history: is absent. Stale allow entry table:missing
            // remains.
            ("allow_stale", root.clone(), config("allow-stale.yml")),
            // "keyed" is absent. Remain: (shape empty) no primary key,
            // (shape triggers) no BEFORE UPDATE FOR EACH ROW trigger,
            // (shape not_fk) owner_id must not be a foreign key,
            // (shape other) foreign key on owner_id must reference,
            // and "no column matches ^absent$ with nullable".
            // (shape quoted) is absent.
            ("column_lists", root.clone(), config("column-lists.yml")),
            // Exact message includes "edges.json: table:empty_key: use a real key".
            ("custom_message", root.clone(), config("custom-message.yml")),
            (
                "include_misses_catalog",
                root.clone(),
                config("include-misses.yml"),
            ),
            // Remain: no column matches ^note$ with type uuid; ^note$ with
            // NOT NULL; ^owner_id$ with not a foreign key; ^absent$ with a
            // foreign key, ON DELETE CASCADE; no AFTER INSERT FOR EACH ROW
            // trigger executing a"b(); no BEFORE TRUNCATE FOR EACH STATEMENT
            // trigger executing fn_trunc().
            (
                "pattern_diagnostics",
                root,
                config("pattern-diagnostics.yml"),
            ),
        ],
    );
}
