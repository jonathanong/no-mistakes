use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn catalog() -> PathBuf {
    rules_path(
        "postgres-status-with-lifecycle-timestamps",
        "../../../fixtures/postgres",
        "status-lifecycle",
    )
}

#[test]
fn postgres_status_with_lifecycle_timestamps_check_json_snapshot() {
    let rule = "postgres-status-with-lifecycle-timestamps";
    let root = catalog();
    let config = |name: &str| rules_path(rule, "configs", name);
    // reports_stored_status_beside_lifecycle_timestamps and
    // valid_shapes_and_non_lifecycle_columns_pass share main. The repeated
    // main_options equality is the jobs 1 vs 2 check on that case.
    // Must contain deliveries/state + sent_at, failed_at; payments/status +
    // payment_failed_at, paid_at; precise_events/status + sent_at, failed_at;
    // both_names/state + started_at, finished_at (not "table stores status
    // column status"); generated_state/state + sent_at, bounced_at (not status
    // column status). Absent tables: invoices, invoice_drafts, receipts,
    // orders, dated_approvals, preview_invoices, resent_jobs, payment_intents,
    // projected_events, event_lists, status_history, named_types.
    assert_saved_fixture(
        "postgres_status_with_lifecycle_timestamps",
        &[
            ("main", root.clone(), config("main.yml")),
            ("no_verbs", root.clone(), config("no-verbs.yml")),
            ("empty_verbs", root.clone(), config("empty-verbs.yml")),
            // Contains deliveries and payments. invoice_drafts is absent.
            (
                "columns_and_verbs",
                root.clone(),
                config("columns-and-verbs.yml"),
            ),
            // payments remains. deliveries is absent.
            ("status_only", root.clone(), config("status-only.yml")),
            // Contains invoice_drafts status next to sent_at.
            ("min_one", root.clone(), config("min-one.yml")),
            // table:deliveries: is absent.
            ("min_three", root.clone(), config("min-three.yml")),
            // table:deliveries: table stores is absent. payments remains.
            (
                "allow_deliveries",
                root.clone(),
                config("allow-deliveries.yml"),
            ),
            // Stale allow entry table:missing remains.
            ("stale_missing", root.clone(), config("stale-missing.yml")),
            // Contains "table:deliveries: use one source of truth".
            ("custom_message", root.clone(), config("custom-message.yml")),
            ("include_misses_catalog", root, config("include-misses.yml")),
        ],
    );
}
