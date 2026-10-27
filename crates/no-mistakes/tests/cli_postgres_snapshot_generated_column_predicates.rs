use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_generated_column_predicates_check_json_snapshot() {
    let rule = "postgres-generated-column-predicates";
    // check_with_files outputs from tests.rs, followup_tests.rs, and
    // history_tests.rs. Directive fixtures are the suppressed CLI outcome.
    let cases = [
        ("invalid_examples", "reported", "sql.yml"),
        ("standalone_table", "standalone", "sql.yml"),
        ("valid_examples", "pass", "sql.yml"),
        // Messages must not contain note_at or ambiguous.
        ("stored_and_virtual", "edges", "sql.yml"),
        ("non_primary_key_argument", "edges", "pk-off.yml"),
        // Messages must not contain ORDER BY.
        ("where_clause_only", "reported", "where-only.yml"),
        // Messages must not contain WHERE filters.
        ("order_clause_only", "reported", "order-only.yml"),
        // Messages must not contain WHERE filters.
        ("join_clause_only", "edges", "join-only.yml"),
        ("stale_extra_column", "reported", "stale.yml"),
        (
            "extra_column_absent_from_schema",
            "extra-invoices",
            "extra-invoices.yml",
        ),
        // Messages must not contain ORDER BY.
        ("include_skips_order", "include-order", "include-order.yml"),
        ("no_tracked_columns", "coverage", "coverage.yml"),
        ("suppressed_predicates", "suppress", "sql.yml"),
        ("unsuppressed_predicates", "suppress-raw", "sql.yml"),
        // Messages must not contain UUIDv7 bound or primary-key index.
        ("lines", "loc-lines", "sql.yml"),
        ("inequality", "loc-inequality", "sql.yml"),
        ("alias", "loc-alias", "sql.yml"),
        ("mutations", "loc-mutations", "sql.yml"),
        ("alter", "loc-alter", "sql.yml"),
        ("qualified", "loc-qualified", "sql.yml"),
        ("join_scope", "loc-join-scope", "sql.yml"),
        ("unknown_relation", "loc-unknown", "sql.yml"),
        ("ext_ts", "ext-ts", "sql.yml"),
        ("ext_ts_include", "ext-ts", "ext-ts.yml"),
        ("ext_mjs", "ext-mjs", "sql.yml"),
        ("ext_mjs_include", "ext-mjs", "ext-mjs.yml"),
        ("ext_cjs", "ext-cjs", "sql.yml"),
        ("ext_cjs_include", "ext-cjs", "ext-cjs.yml"),
        ("ext_jsx", "ext-jsx", "sql.yml"),
        ("ext_jsx_include", "ext-jsx", "ext-jsx.yml"),
        ("ext_cts", "ext-cts", "sql.yml"),
        ("ext_cts_include", "ext-cts", "ext-cts.yml"),
        // Messages must not contain JOIN ON compares.
        (
            "clauses_without_join",
            "clause-join-inputs",
            "where-order.yml",
        ),
        // Messages must not contain uuid_extract_timestamp(id).
        (
            "extra_column_without_schema",
            "extra-events",
            "extra-events.yml",
        ),
        ("parse_error_line", "parse-error", "sql.yml"),
        ("review_suppress_sql", "review-suppress-sql-raw", "sql.yml"),
        (
            "review_suppress_sql_hidden",
            "review-suppress-sql",
            "sql.yml",
        ),
        ("review_suppress_ts", "review-suppress-ts-raw", "sql.yml"),
        ("review_suppress_ts_hidden", "review-suppress-ts", "sql.yml"),
        // Messages must not contain UUIDv7, same order, or primary-key index.
        (
            "arbitrary_generation_function",
            "custom-fn",
            "custom-fn.yml",
        ),
        ("quoted_names", "quoted-names", "sql.yml"),
        ("dynamic_call", "dynamic-owner", "sql.yml"),
        ("history_alter", "history-alter", "sql.yml"),
        ("history_drop", "history-drop", "sql.yml"),
        ("history_recreate", "history-recreate", "sql.yml"),
        ("history_same_file", "history-same-file", "sql.yml"),
        ("prepared_reuse", "prepared-reuse", "prepared-reuse.yml"),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(rule, "fixture", fixture),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_generated_column_predicates", &cases);
}
