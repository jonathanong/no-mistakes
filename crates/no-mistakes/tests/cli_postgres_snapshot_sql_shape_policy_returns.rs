use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_sql_shape_policy_returns_check_json_snapshot() {
    let rule = "postgres-sql-shape-policy";
    // Output cases formerly asserted by the rule's unit tests.
    // The snapshot keeps counts, messages, targets, files, lines, and clean reports.
    let cases = [
        // CLI keeps lines 4, 14, 23, 26, 31, 33, and 36. The unit test still sees raw lines 10, 18, and 39 before suppression.
        (
            "implicit_fetch_origins_suppressed",
            "fixture",
            "review-followups",
            "implicit_fetch_origins_suppressed.yml",
        ),
        // CLI keeps keyset-only-sweep on table-pages.sql lines 2, 3, 4, 5, and 10, and table-pages.ts line 2. The unit test still sees raw sql line 14 and ts line 4 before suppression.
        (
            "table_pages_suppressed",
            "fixture",
            "review-followups",
            "table_pages_suppressed.yml",
        ),
        // Lines 1 through 12, all banned-function-call. Identifiers that are not calls stay clean.
        (
            "banned_functions_positions",
            "fixture",
            "banned-functions",
            "banned_functions_positions.yml",
        ),
        // Lines 1, 2, and 3.
        (
            "banned_name_pg_sleep",
            "fixture",
            "banned-functions",
            "banned_name_pg_sleep.yml",
        ),
        // Line 2 only.
        (
            "banned_name_public_pg_sleep",
            "fixture",
            "banned-functions",
            "banned_name_public_pg_sleep.yml",
        ),
        // Line 4 only. The configured name keeps the quotes and case.
        (
            "banned_name_quoted_pg_sleep",
            "fixture",
            "banned-functions",
            "banned_name_quoted_pg_sleep.yml",
        ),
        // Line 5 only.
        (
            "banned_name_quoted_odd_function",
            "fixture",
            "banned-functions",
            "banned_name_quoted_odd_function.yml",
        ),
        // Line 6 only.
        (
            "banned_name_custom_other",
            "fixture",
            "banned-functions",
            "banned_name_custom_other.yml",
        ),
        // Line 7 only.
        (
            "banned_name_custom_other_quoted",
            "fixture",
            "banned-functions",
            "banned_name_custom_other_quoted.yml",
        ),
        // Lines 2, 3, 4, 5, and 6.
        (
            "banned_functions_embedded",
            "fixture",
            "banned-functions",
            "banned_functions_embedded.yml",
        ),
        // Lines 2, 3, 4, 5, 8, 9, 12, and 13.
        (
            "banned_functions_routines",
            "fixture",
            "banned-functions",
            "banned_functions_routines.yml",
        ),
        // Lines 2, 5, 6, 7, 8, and 11.
        (
            "banned_functions_procedural",
            "fixture",
            "banned-functions",
            "banned_functions_procedural.yml",
        ),
        // Every finding target is banned-function-call.
        (
            "banned_functions_dynamic",
            "fixture",
            "banned-functions",
            "banned_functions_dynamic.yml",
        ),
        // CLI suppression of both disabled SQL files is empty. The unit test still sees 3 findings before suppression.
        (
            "banned_functions_disabled",
            "fixture",
            "banned-functions",
            "banned_functions_disabled.yml",
        ),
        // 10 findings. Every message ends with 'is banned in WHERE; stable bound'. Identical call diagnostics on one line are deduped.
        (
            "clause_where",
            "fixture",
            "clause-policy",
            "clause_where.yml",
        ),
        // 2 findings. Every message ends with 'is banned in JOIN ON; stable bound'. Identical call diagnostics on one line are deduped.
        (
            "clause_join_on",
            "fixture",
            "clause-policy",
            "clause_join_on.yml",
        ),
        // 1 finding. The message ends with 'is banned in HAVING; stable bound'. Identical call diagnostics on one line are deduped.
        (
            "clause_having",
            "fixture",
            "clause-policy",
            "clause_having.yml",
        ),
        // 5 findings. Every message ends with 'is banned in SELECT list; stable bound'. Identical call diagnostics on one line are deduped.
        (
            "clause_select_list",
            "fixture",
            "clause-policy",
            "clause_select_list.yml",
        ),
        // 1 finding. The message ends with 'is banned in ORDER BY; stable bound'. Identical call diagnostics on one line are deduped.
        (
            "clause_order_by",
            "fixture",
            "clause-policy",
            "clause_order_by.yml",
        ),
        // 2 findings. Every message ends with 'is banned in VALUES; stable bound'. Identical call diagnostics on one line are deduped.
        (
            "clause_values",
            "fixture",
            "clause-policy",
            "clause_values.yml",
        ),
        // 3 findings. Every message ends with 'is banned in SET; stable bound'. Identical call diagnostics on one line are deduped.
        ("clause_set", "fixture", "clause-policy", "clause_set.yml"),
        // 2 findings. Every message ends with 'is banned in DEFAULT; stable bound'. Identical call diagnostics on one line are deduped.
        (
            "clause_default",
            "fixture",
            "clause-policy",
            "clause_default.yml",
        ),
        // 4 findings. Every message ends with 'is banned in RETURNING; stable bound'. Identical call diagnostics on one line are deduped.
        (
            "clause_returning",
            "fixture",
            "clause-policy",
            "clause_returning.yml",
        ),
        // String form [uuidv7] and clause_object_uuidv7 must report the same diagnostics.
        (
            "clause_string_uuidv7",
            "fixture",
            "clause-policy",
            "clause_string_uuidv7.yml",
        ),
        // Object form without clauses must match clause_string_uuidv7.
        (
            "clause_object_uuidv7",
            "fixture",
            "clause-policy",
            "clause_object_uuidv7.yml",
        ),
        // Every message ends with '; choose stable'.
        (
            "clause_hinted_uuidv7",
            "fixture",
            "clause-policy",
            "clause_hinted_uuidv7.yml",
        ),
        // One finding on line 14.
        (
            "qualified_uuidv7_where",
            "fixture",
            "clause-policy",
            "qualified_uuidv7_where.yml",
        ),
        // Lines 2 and 3.
        (
            "review_uuidv7_where",
            "fixture",
            "clause-policy",
            "review_uuidv7_where.yml",
        ),
        // Three findings.
        (
            "review_uuidv7_global",
            "fixture",
            "clause-policy",
            "review_uuidv7_global.yml",
        ),
        // One finding whose message ends with "custom.schema"."odd.function"() is banned in WHERE.
        (
            "review_quoted_odd_function",
            "fixture",
            "clause-policy",
            "review_quoted_odd_function.yml",
        ),
        // One finding whose message ends with "Upper"."quote""function"() is banned in WHERE.
        (
            "review_quoted_upper_function",
            "fixture",
            "clause-policy",
            "review_quoted_upper_function.yml",
        ),
        // One finding whose message ends with _simple$9() is banned in WHERE.
        (
            "review_simple_function",
            "fixture",
            "clause-policy",
            "review_simple_function.yml",
        ),
        // One finding whose message ends with "select"() is banned in WHERE.
        (
            "review_quoted_select_function",
            "fixture",
            "clause-policy",
            "review_quoted_select_function.yml",
        ),
        // One literal-limit on line 3. The private fragment-line assert stays in the unit test.
        (
            "returned_named",
            "fixture",
            "returned-call-identities",
            "returned_named.yml",
        ),
        // One literal-limit on line 3. The private fragment-line assert stays in the unit test.
        (
            "returned_await",
            "fixture",
            "returned-call-identities",
            "returned_await.yml",
        ),
        // One literal-limit on line 5. The private fragment-line assert stays in the unit test.
        (
            "returned_shadow",
            "fixture",
            "returned-call-identities",
            "returned_shadow.yml",
        ),
        // One literal-limit on line 6. The private fragment-line assert stays in the unit test.
        (
            "returned_recursive",
            "fixture",
            "returned-call-identities",
            "returned_recursive.yml",
        ),
        // One literal-limit on line 5. The private fragment-line assert stays in the unit test.
        (
            "returned_shadow_edge",
            "fixture",
            "returned-call-identities",
            "returned_shadow_edge.yml",
        ),
        // Lines 5 and 6. The unused outer non-SQL return on line 3 must not be a finding.
        (
            "shadowed_call_fail_closed",
            "fixture",
            "returned-call-identities",
            "shadowed_call_fail_closed.yml",
        ),
        // One literal-limit on line 4. The private fragment-line assert stays in the unit test.
        (
            "member_class_method",
            "fixture",
            "returned-member-owners",
            "member-class-method.yml",
        ),
        // One literal-limit on line 4. The private fragment-line assert stays in the unit test.
        (
            "member_class_field_arrow",
            "fixture",
            "returned-member-owners",
            "member-class-field-arrow.yml",
        ),
        // One literal-limit on line 4. The private fragment-line assert stays in the unit test.
        (
            "member_this_method",
            "fixture",
            "returned-member-owners",
            "member-this-method.yml",
        ),
        // One literal-limit on line 4. The private fragment-line assert stays in the unit test.
        (
            "member_object_method",
            "fixture",
            "returned-member-owners",
            "member-object-method.yml",
        ),
        // One literal-limit on line 4. The private fragment-line assert stays in the unit test.
        (
            "member_property_function",
            "fixture",
            "returned-member-owners",
            "member-property-function.yml",
        ),
        // One literal-limit on line 4. The private fragment-line assert stays in the unit test.
        (
            "member_named_property_function",
            "fixture",
            "returned-member-owners",
            "member-named-property-function.yml",
        ),
        // One literal-limit on line 4. The private fragment-line assert stays in the unit test.
        (
            "member_unused_method",
            "fixture",
            "returned-member-owners",
            "member-unused-method.yml",
        ),
        // One literal-limit on line 4. The private fragment-line assert stays in the unit test.
        (
            "member_method_forwarding",
            "fixture",
            "returned-member-owners",
            "member-method-forwarding.yml",
        ),
        // Lines 2 and 3. Duplicate executor names canonicalize. Missing prepared facts stay unit-test errors.
        (
            "prepared_custom_executors",
            "fixture",
            "prepared",
            "prepared_custom_executors.yml",
        ),
        // CLI keeps lines 6 and 10. The unit test still sees raw lines 6, 10, 16, and 20 before suppression.
        (
            "from_less_builders_suppressed",
            "fixture",
            "prepared",
            "from_less_builders_suppressed.yml",
        ),
        // CLI hides the disable-next-line call on line 7. The unit test still expects raw lines 2 and 7, both banned-function-call.
        (
            "executed_complete_owned",
            "variants",
            "executed-complete-owned",
            "executed_complete_owned.yml",
        ),
        // CLI suppression is empty. The unit test still expects raw line 7 for suppression accounting, and the fragment finding must not leak.
        (
            "complete_suppression_owned",
            "variants",
            "complete-suppression-owned",
            "executed_complete_owned.yml",
        ),
        // CLI hides the disabled call. The unit test still expects raw lines 2 and 5: the unrelated builder stays on the token line.
        (
            "complete_suppression_reuse",
            "variants",
            "complete-suppression-reuse",
            "executed_complete_owned.yml",
        ),
        // CLI hides the disabled host. The unit test still expects a raw banned-function-call on line 4.
        (
            "host_suppressed_finite_fragments",
            "variants",
            "finite-fragments-host-suppressed",
            "executed_complete_owned.yml",
        ),
        // Two findings. Line 2's message starts with src/query.ts:2:. Line 10's message starts with 'src/query.ts:10: builder SQL'.
        (
            "finite_fragments_unexecuted",
            "variants",
            "finite-fragments-unexecuted",
            "executed_complete_owned.yml",
        ),
    ]
    .map(|(label, kind, fixture, config)| {
        (
            label,
            rules_path(rule, kind, fixture),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_sql_shape_policy_returns", &cases);
}
