use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_no_transformed_uuid_predicates_check_json_snapshot() {
    let rule = "postgres-no-transformed-uuid-predicates";
    // These low/max pairs match: embedded_literals, custom_message, suppressed,
    // operand_pass, operand_positive, operand_ts, aggregate_pass,
    // aggregate_positive, aggregate_ts, and crlf. Policy fail/ignore pairs
    // match across effort, as do ambiguous and invalid advise pairs.
    // `policy_invalid_ts_advise_low` names effort low; max says invalid-sql.
    // Both have empty rules. `verdict_inputs_advise` rules match; low also
    // reports unresolved advisories on lines 5 and 6. SqlVerdictStatus stays
    // in the unit test. Advise keeps `rules` empty. Ignore is empty rules and
    // empty advisories.
    let rows = [
        // Nine findings. Lines 1 WHERE, 4 JOIN ON, 5 HAVING, 6 WHERE, and 7 WHERE
        // must remain. Every message contains "Cast the parameter to uuid" and
        // "precomputed bound".
        ("fail_clauses", "fail-clauses.yml"),
        ("bare_columns", "bare-columns.yml"),
        // Lines 1, 2, and 11 must remain.
        ("catalog_resolution", "scope-ignore.yml"),
        // Line 4 target unanalyzable must remain.
        ("unknown_owners", "scope-default.yml"),
        // Lines 2, 3, and 5 must remain.
        ("embedded_literals", "embedded.yml"),
        ("embedded_literals_low", "embedded-low.yml"),
        ("unrecoverable_embedded", "unknown.yml"),
        ("unrecoverable_embedded_ignore", "unknown-ignore.yml"),
        // Every message starts with "Keep identity lookups indexable" and contains
        // "UUID column" and "Cast the parameter".
        ("custom_message", "custom-message.yml"),
        ("custom_message_low", "custom-message-low.yml"),
        // fail.sql keeps 9 findings and embedded.ts keeps 3.
        ("sql_include_independent", "sql-include-independent.yml"),
        // The disable comment hides the executor call.
        ("suppressed", "suppressed.yml"),
        ("suppressed_low", "suppressed-low.yml"),
        // Lines 1, 3, 4, 5, and 6 must remain.
        ("quoted", "quoted.yml"),
        // Lines 2, 5, 6, 7, and 8 must remain.
        ("ctes", "ctes.yml"),
        // Seven findings, every target unanalyzable.
        ("unknown_scope", "unknown-scope.yml"),
        ("unknown_scope_ignore", "unknown-scope-ignore.yml"),
        // Seven findings, and none are target unanalyzable.
        ("predicate_forms", "predicate-forms.yml"),
        ("exclude_sql", "exclude-sql.yml"),
        ("exclude_ts", "exclude-ts.yml"),
        // Lines 1, 2, 2, 3, 4, and 4 must remain, and five messages contain "JOIN ON".
        ("nested_joins", "nested-joins.yml"),
        ("operand_pass_low", "operand-pass-low.yml"),
        ("operand_pass_max", "operand-pass-max.yml"),
        // Lines 2 through 13, every target accounts.id.
        ("operand_positive_low", "operand-positive-low.yml"),
        ("operand_positive_max", "operand-positive-max.yml"),
        // Lines 8, 9, 10, and 11 must remain.
        ("operand_ts_low", "operand-ts-low.yml"),
        ("operand_ts_max", "operand-ts-max.yml"),
        ("aggregate_pass_low", "aggregate-pass-low.yml"),
        ("aggregate_pass_max", "aggregate-pass-max.yml"),
        // Lines 2 through 11 and 14 through 17. Lines 9 through 11 target
        // accounts.token; the others target accounts.id.
        ("aggregate_positive_low", "aggregate-positive-low.yml"),
        ("aggregate_positive_max", "aggregate-positive-max.yml"),
        // Lines 10 through 15. Line 14 and later target accounts.token; earlier lines target accounts.id.
        ("aggregate_ts_low", "aggregate-ts-low.yml"),
        ("aggregate_ts_max", "aggregate-ts-max.yml"),
        // Lines 4 and 6 must remain. crlf_max is the default effort.
        ("crlf_low", "crlf-low.yml"),
        ("crlf_max", "crlf-max.yml"),
        // Two rule findings. Advise may also record advisories.
        ("verdict_inputs_advise_low", "verdict-inputs-advise-low.yml"),
        ("verdict_inputs_advise_max", "verdict-inputs-advise-max.yml"),
        ("recovered_parameters", "recovered-parameters.yml"),
        // One finding on line 2.
        ("values", "values.yml"),
        // One rule finding on line 5, target accounts.id. Other uncertainty is advisory.
        ("low_inputs_advise", "low-inputs-advise.yml"),
        // Lines 2, 4, and 5. Two are unanalyzable, and line 5 targets accounts.id.
        ("low_inputs_fail", "low-inputs-fail.yml"),
        // Six findings, four of them unanalyzable.
        ("verdict_inputs_low_fail", "verdict-inputs-low-fail.yml"),
        ("policy_invalid_fail_low", "policy-invalid-fail-low.yml"),
        ("policy_invalid_fail_max", "policy-invalid-fail-max.yml"),
        ("policy_invalid_ignore_low", "policy-invalid-ignore-low.yml"),
        ("policy_invalid_ignore_max", "policy-invalid-ignore-max.yml"),
        ("policy_invalid_advise_low", "policy-invalid-advise-low.yml"),
        ("policy_invalid_advise_max", "policy-invalid-advise-max.yml"),
        ("policy_ambiguous_fail_low", "policy-ambiguous-fail-low.yml"),
        ("policy_ambiguous_fail_max", "policy-ambiguous-fail-max.yml"),
        (
            "policy_ambiguous_ignore_low",
            "policy-ambiguous-ignore-low.yml",
        ),
        (
            "policy_ambiguous_ignore_max",
            "policy-ambiguous-ignore-max.yml",
        ),
        (
            "policy_ambiguous_advise_low",
            "policy-ambiguous-advise-low.yml",
        ),
        (
            "policy_ambiguous_advise_max",
            "policy-ambiguous-advise-max.yml",
        ),
        // Fail findings are non-empty. Advise leaves rules empty.
        (
            "policy_invalid_ts_fail_low",
            "policy-invalid-ts-fail-low.yml",
        ),
        (
            "policy_invalid_ts_fail_max",
            "policy-invalid-ts-fail-max.yml",
        ),
        (
            "policy_invalid_ts_advise_low",
            "policy-invalid-ts-advise-low.yml",
        ),
        (
            "policy_invalid_ts_advise_max",
            "policy-invalid-ts-advise-max.yml",
        ),
    ];
    let root = rules_path(rule, "fixture", ".");
    let cases =
        rows.map(|(label, config)| (label, root.clone(), rules_path(rule, "configs", config)));
    assert_saved_fixture("postgres_no_transformed_uuid_predicates", &cases);
}
