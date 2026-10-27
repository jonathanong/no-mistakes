use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_identifier_length_check_json_snapshot() {
    let rule = "postgres-identifier-length";
    let limit = "max-bytes-63.yml";
    let cases = [
        ("pass_63", "pass", "migrations.yml"),
        // referenced_only must not appear. do_index is one finding.
        ("declared_names", "fail", "migrations.yml"),
        ("multibyte_cut", "multibyte", limit),
        ("quoted_unescape", "quotes", limit),
        ("duplicate_name", "duplicate", limit),
        ("references_ignored", "reference", limit),
        // Messages must not contain app.
        ("schema_qualifier", "schema-qualified", limit),
        ("do_block_line", "do-block", limit),
        ("function_body_line", "function-body", limit),
        ("max_bytes_10", "max-bytes", "max-bytes-10.yml"),
        ("max_bytes_11", "max-bytes", "max-bytes-11.yml"),
        ("sql_include_skips", "fail", "skip.yml"),
        ("typescript_out_of_scope", "ignored-ts", "ignored.yml"),
        // repeated_table stays on lines 4 and 7. decoy_table must not appear.
        // temporary_table is line 10, unlogged_table 11, after_invalid 13,
        // replaced_view and explicit_column 14, temporary_view 15,
        // replaced_trigger 16, constraint_trigger 18.
        (
            "declaration_locations",
            "declaration-locations",
            "max-bytes-10.yml",
        ),
        // commented_procedure line 2, escaped_procedure line 3,
        // custom_procedure line 4, dynamic_procedure line 7.
        ("procedure_tokens", "procedure-tokens", "max-bytes-10.yml"),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(rule, "fixture", fixture),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_identifier_length", &cases);
}
