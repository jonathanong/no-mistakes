use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn repo_rel(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
}

fn scenario(name: &str) -> PathBuf {
    repo_rel(&format!(
        "fixtures/postgres/finite-text/unit/scenarios/{name}"
    ))
}

#[test]
fn postgres_finite_text_columns_check_json_snapshot() {
    let rule = "postgres-finite-text-columns";
    let config = |name| rules_path(rule, "configs", name);
    let catalog = config("catalog.yml");
    let names = config("names.yml");
    let cases = [
        // Messages must not contain foreign_status, generated_status, or column:accounts.malformed_state.
        (
            "review_followups",
            repo_rel("fixtures/postgres/finite-text/review-followups"),
            config("review.yml"),
        ),
        ("shared_values", scenario("shared-values"), catalog.clone()),
        (
            "orders_nullable",
            scenario("orders-nullable"),
            catalog.clone(),
        ),
        (
            "payments_single",
            scenario("payments-single"),
            catalog.clone(),
        ),
        ("jobs_scope", scenario("jobs-scope"), catalog.clone()),
        ("shipments", scenario("shipments"), catalog.clone()),
        (
            "composite_foreign_keys",
            scenario("composite-foreign-keys"),
            names.clone(),
        ),
        (
            "valid_columns",
            scenario("valid-columns"),
            config("valid-columns.yml"),
        ),
        (
            "labels_varying",
            scenario("labels-varying"),
            catalog.clone(),
        ),
        ("labels_quoted", scenario("labels-quoted"), catalog.clone()),
        // Messages must not contain "name matches".
        ("pinned_name", scenario("pinned-name"), names.clone()),
        ("null_kind", scenario("null-kind"), catalog.clone()),
        ("null_kind_name", scenario("null-kind"), names.clone()),
        ("generated", scenario("generated"), catalog.clone()),
        (
            "skip_generated",
            scenario("generated"),
            config("skip-generated.yml"),
        ),
        (
            "empty_name_patterns",
            scenario("plan-type-no-name"),
            config("empty-name-patterns.yml"),
        ),
        (
            "ignore_archive",
            scenario("archive-ignored"),
            config("ignore-archive.yml"),
        ),
        ("partitioned", scenario("partitioned"), catalog.clone()),
        ("peer_pair", scenario("peer-pair"), catalog.clone()),
        ("peer_multiple", scenario("peer-multiple"), catalog.clone()),
        ("long_values", scenario("long-values"), catalog.clone()),
        ("long_peers", scenario("long-peers"), catalog.clone()),
        (
            "allow_stale",
            scenario("allow-stale"),
            config("allow-stale.yml"),
        ),
        ("two_checks", scenario("two-checks"), catalog.clone()),
        (
            "prepared_catalog",
            repo_rel("fixtures/postgres/finite-text/pass"),
            catalog.clone(),
        ),
        (
            "custom_message",
            repo_rel("fixtures/postgres/finite-text/review-followups"),
            config("custom-message.yml"),
        ),
        (
            "include_misses_catalog",
            repo_rel("fixtures/postgres/finite-text/review-followups"),
            config("include-missing.yml"),
        ),
        (
            "stale_custom_message",
            scenario("stale-custom-message"),
            config("stale-custom-message.yml"),
        ),
        // column:invoices.status must not appear.
        (
            "line_disable",
            rules_path(rule, "fixture", "line-disable"),
            catalog.clone(),
        ),
        // column:invoices.status must not appear.
        (
            "next_line_disable",
            rules_path(rule, "fixture", "next-line-disable"),
            catalog.clone(),
        ),
        // File disable must leave no findings.
        (
            "file_disable",
            rules_path(rule, "fixture", "file-disable"),
            catalog,
        ),
    ];
    assert_saved_fixture("postgres_finite_text_columns", &cases);
}
