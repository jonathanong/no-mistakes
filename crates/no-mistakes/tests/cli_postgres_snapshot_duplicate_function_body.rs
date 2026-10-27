use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_duplicate_function_body_check_json_snapshot() {
    let rule = "postgres-duplicate-function-body";
    // Output cases formerly asserted by the rule unit tests.
    let cases = [
        // fn_reject_orders_update, fn_reject_invoices_update, and fn_reject_refunds_update must remain
        ("copied_raise_cluster", "rejects", "catalog.yml"),
        // must not contain fn_touch_orders, fn_touch_invoices, fn_sql_copy, fn_plpgsql_copy, fn_reject_mutation, fn_flow_if, fn_flow_case, fn_bad, or fn_nobody
        ("valid_shapes", "schema", "catalog.yml"),
        ("min_tokens_splits_rejects", "rejects", "min-tokens.yml"),
        (
            "normalize_raise_off_splits_rejects",
            "rejects",
            "normalize-raise-off.yml",
        ),
        // must not contain function:fn_title: or function:fn_body:; fn_semi_a must remain
        (
            "normalize_identifiers_off",
            "schema",
            "normalize-identifiers-off.yml",
        ),
        // fn_01 must list fn_02 through fn_06 and 1 more
        ("cluster_above_five", "many", "catalog.yml"),
        // must not contain function:fn_reject_orders_update:; fn_reject_invoices_update and stale fn_orders_audit must remain
        ("allow_suppresses_and_stale", "rejects", "allow-stale.yml"),
        // fn_event_a and event trigger function must remain; must not contain fn_void_same or fn_row_trigger
        ("trigger_events_separate", "kinds", "catalog.yml"),
        // fn_quote_a and fn_quote_b must remain
        ("quoted_bodies", "quoted", "catalog.yml"),
        ("event_default_string", "event-default", "catalog.yml"),
        // fn_definer_sql and fn_definer_sql_copy must remain; must not contain fn_invoker_sql
        ("security_definer_in_body", "security-body", "catalog.yml"),
        // the fn_immutable message must contain fn_immutable_copy and must not contain fn_volatile
        ("volatility_splits_copies", "volatility", "catalog.yml"),
        // the fn_int message must contain fn_int_copy and must not contain fn_bigint; must not contain function:fn_bigint:
        ("return_contract_splits_copies", "returns", "catalog.yml"),
        // the fn_left message must contain fn_left_copy and must not contain fn_right
        ("named_arguments_split_copies", "named", "catalog.yml"),
        ("comment_security", "comment-security", "catalog.yml"),
        // the fn_quoted_a message must contain fn_quoted_a_copy and must not contain fn_quoted_lower
        ("quoted_search_path", "quoted-path", "catalog.yml"),
        ("concat_bodies", "concat", "catalog.yml"),
        // fn_ret_a and fn_ret_b must remain
        ("return_expressions", "return-expr", "catalog.yml"),
        // fn_escape_a and fn_escape_b must remain
        ("escape_strings", "escape", "catalog.yml"),
        // fn_atomic_a and fn_atomic_b must remain
        ("atomic_language", "atomic", "catalog.yml"),
        // the fn_strict message must contain fn_strict_long and must not contain fn_called or fn_default_word; function:fn_called: and function:fn_default_word: must remain
        ("strict_not_called_on_null", "strict", "catalog.yml"),
        // fn_definer and fn_definer_copy must remain; must not contain fn_invoker
        ("security_definer_not_invoker", "security", "catalog.yml"),
        // fn_path must remain; must not contain function:fn_definer:
        ("security_path_clause", "security-path", "catalog.yml"),
        // fn_if must remain; must not contain function:fn_other:
        ("atomic_if_clause", "atomic-if", "catalog.yml"),
        ("dollar_path", "dollar-path", "catalog.yml"),
        ("out_params", "out-params", "catalog.yml"),
        // fn_tenant_a must remain; must not contain function:fn_tenant_b:
        ("newline_search_path", "newline-path", "catalog.yml"),
        ("quoted_return", "quoted-return", "catalog.yml"),
        ("hex_escape", "hex-escape", "catalog.yml"),
        ("kept_identifier", "kept-name", "keep-mixed-case.yml"),
        // fn_plain must remain; must not contain function:fn_leak:
        ("leakproof_splits", "leakproof", "catalog.yml"),
        // fn_int must remain; must not contain function:fn_array:
        ("array_return_splits", "arrays", "catalog.yml"),
        // fn_safe must remain; must not contain function:fn_unsafe:
        ("parallel_splits", "parallel", "catalog.yml"),
        ("escape_default", "escape-default", "catalog.yml"),
        // fn_tenant_a and fn_tenant_a_copy must remain; must not contain function:fn_tenant_b:
        ("search_path_splits", "search-path", "catalog.yml"),
        ("include_skips_catalog", "rejects", "include-api.yml"),
        // merge the copies must remain; must not contain TG_TABLE_NAME
        ("custom_message", "many", "merge-message.yml"),
        // fn_param and fn_param_copy must remain; must not contain fn_real or fn_c
        ("parameter_names_not_volatility", "modes", "catalog.yml"),
        // fn_alpha, fn_alpha_copy, fn_cafe, and fn_cafe_copy must remain; must not contain fn_beta or fn_public
        ("unicode_and_search_path", "unicode", "catalog.yml"),
        // fn_set, fn_text, fn_send, fn_atomic, and fn_u must remain; must not contain fn_real_strict, fn_int, fn_delete, or fn_string
        ("clause_boundaries", "boundaries", "catalog.yml"),
        // fn_table_int, fn_rows_one, fn_op_a, fn_support_one, and fn_bang must remain; must not contain fn_table_text, fn_rows_many, fn_op_b, fn_support_two, or fn_cang
        ("planner_estimates", "planner", "catalog.yml"),
        // fn_lang_sql, fn_atomic_gap, fn_path_stable, fn_cost_low, fn_eend, fn_type_stable, and fn_outer must remain; must not contain fn_lang_c, fn_atomic_string, fn_path_immutable, fn_cost_high, fn_eend_other, fn_type_volatile, or fn_outer_other
        ("comments_and_costs", "gaps", "catalog.yml"),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(rule, "fixture", fixture),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_duplicate_function_body", &cases);
}
