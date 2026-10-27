use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn repo_rel(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
}

#[test]
fn postgres_bounded_statements_check_json_snapshot() {
    let cases = [
        // three findings; direct and shared entry points must agree
        ("invalid_sql", "test-cases/rules/postgres-bounded-statements/fixture", "invalid_sql.yml"),
        // target unanalyzable; messages must mention analyzed and recoverable
        ("unanalyzable_fail", "test-cases/rules/postgres-bounded-statements/fixture", "unanalyzable_fail.yml"),
        // unanalyzableSql ignore must stay empty
        ("unanalyzable_ignore", "test-cases/rules/postgres-bounded-statements/fixture", "unanalyzable_ignore.yml"),
        // line directives must suppress their statements; only unsuppressed statement lines remain
        ("statement_suppression", "test-cases/rules/postgres-bounded-statements/fixture", "statement_suppression.yml"),
        // after suppression only the unsuppressed write lines remain (17 and 19)
        ("wrapped_statement_suppression", "test-cases/rules/postgres-bounded-statements/fixture", "wrapped_statement_suppression.yml"),
        // embedded directives; survivors are the calls without a matching directive (lines 9 and 11)
        ("wrapped_embedded_suppression", "test-cases/rules/postgres-bounded-statements/fixture", "wrapped_embedded_suppression.yml"),
        // line 16 must be absent after suppression; line 4 must remain; lines 2 3 7 8 9 10 14 17 must be absent
        ("rejecting_having_suppression", "test-cases/rules/postgres-bounded-statements/fixture", "rejecting_having_suppression.yml"),
        // line 17 must be absent after suppression; line 3 must remain
        ("pin_query_suppression", "test-cases/rules/postgres-bounded-statements/fixture", "pin_query_suppression.yml"),
        // first statement stays bounded under catalog.json; ambiguous owners are a separate case
        ("catalog_resolution_key_resolution", "test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution", "catalog_resolution_key_resolution.yml"),
        ("catalog_resolution_key_operators", "test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution", "catalog_resolution_key_operators.yml"),
        // only view:public.unbounded_archive on line 3
        ("catalog_resolution_view_bounds", "test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution", "catalog_resolution_view_bounds.yml"),
        // advisory policy must not emit a bounded-statement finding
        ("catalog_resolution_view_unconfigured_advise", "test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution", "catalog_resolution_view_unconfigured_advise.yml"),
        // ignore policy must not emit a bounded-statement finding
        ("catalog_resolution_view_unconfigured_ignore", "test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution", "catalog_resolution_view_unconfigured_ignore.yml"),
        // fail policy still has no bounded-statement finding for the unknown view
        ("catalog_resolution_view_unconfigured_fail", "test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution", "catalog_resolution_view_unconfigured_fail.yml"),
        // must stay empty
        ("catalog_resolution_view_allow", "test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution", "catalog_resolution_view_allow.yml"),
        // must stay empty
        ("catalog_resolution_view_suppression", "test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution", "catalog_resolution_view_suppression.yml"),
        // one finding for table:public.customer_accounts
        ("catalog_resolution_key_booleans", "test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution", "catalog_resolution_key_booleans.yml"),
        // target must be view:archive.active_accounts on line 1
        ("catalog_resolution_identity_view_identity", "test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution-identity", "catalog_resolution_identity_view_identity.yml"),
        // must stay empty
        ("catalog_resolution_identity_archive_schema_view_policy", "test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution-identity", "catalog_resolution_identity_archive_schema_view_policy.yml"),
        // public.customer_accounts and public.owner_jobs must both remain
        ("key_operators_ambiguous_owner", "test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution", "key_operators_ambiguous_owner.yml"),
        // ambiguous owners must report public.customer_accounts and public.owner_jobs; the unique catalog case stays empty
        ("key_resolution_ambiguous_owner", "test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution", "key_resolution_ambiguous_owner.yml"),
        // must stay bounded
        ("quoted_order_id_bounded", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "quoted_order_id_bounded.yml"),
        // mixed must remain
        ("quoted_order_id_unbounded", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "quoted_order_id_unbounded.yml"),
        // accounts must remain
        ("current_database_mixed_view_audit", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_mixed_view_audit.yml"),
        // must stay bounded for database other
        ("current_database_mixed_view_other", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_mixed_view_other.yml"),
        // unbounded statement indexes differ by database; Audit.Database
        ("current_database_identity_audit", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_identity_audit.yml"),
        // database other must report more unbounded statements than Audit.Database
        ("current_database_identity_other", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_identity_other.yml"),
        // omitted currentDatabase matches the other-database findings
        ("current_database_identity_absent", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_identity_absent.yml"),
        ("current_database_views_audit", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_views_audit.yml"),
        // folded name auditdb
        ("current_database_views_auditdb", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_views_auditdb.yml"),
        // AuditDB must not fold onto Audit.Database
        ("current_database_views_auditdb_folded", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_views_auditdb_folded.yml"),
        // searchPathEvidence public accounts; statements 1 and 3 unbounded
        ("current_database_search_path_audit", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_search_path_audit.yml"),
        // currentDatabase other must leave every statement unbounded
        ("current_database_search_path_other", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_search_path_other.yml"),
        // parse may fail; recovered bounds still judged
        ("current_database_partitions_audit", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_partitions_audit.yml"),
        ("current_database_partitions_other", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_partitions_other.yml"),
        ("current_database_materialized_audit", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_materialized_audit.yml"),
        ("current_database_materialized_other", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "current_database_materialized_other.yml"),
        // tables that must remain: partial_docs
        ("partial_unique_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "partial_unique_2.yml"),
        // tables that must remain: partial_docs
        ("partial_unique_3", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "partial_unique_3.yml"),
        // tables that must remain: partial_docs
        ("partial_unique_4", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "partial_unique_4.yml"),
        // tables that must remain: partial_docs
        ("partial_unique_5", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "partial_unique_5.yml"),
        // tables that must remain: partial_docs
        ("partial_unique_6", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "partial_unique_6.yml"),
        // these statements must stay bounded
        ("partial_unique_bounded", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "partial_unique_bounded.yml"),
        // tables that must remain: normalized_emails
        ("expression_unique_3", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "expression_unique_3.yml"),
        // tables that must remain: normalized_emails
        ("expression_unique_4", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "expression_unique_4.yml"),
        // tables that must remain: normalized_emails
        ("expression_unique_6", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "expression_unique_6.yml"),
        // these statements must stay bounded
        ("expression_unique_bounded", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "expression_unique_bounded.yml"),
        // tables that must remain: partial_emails
        ("partial_expression_unique_1", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "partial_expression_unique_1.yml"),
        // tables that must remain: partial_emails
        ("partial_expression_unique_3", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "partial_expression_unique_3.yml"),
        // tables that must remain: partial_emails
        ("partial_expression_unique_4", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "partial_expression_unique_4.yml"),
        // tables that must remain: partial_emails
        ("partial_expression_unique_5", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "partial_expression_unique_5.yml"),
        // these statements must stay bounded
        ("partial_expression_unique_bounded", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "partial_expression_unique_bounded.yml"),
        // tables that must remain: partial_logins
        ("partial_column_nullguard_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "partial_column_nullguard_2.yml"),
        // these statements must stay bounded
        ("partial_column_nullguard_bounded", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "partial_column_nullguard_bounded.yml"),
        // tables that must remain: plain_contacts
        ("nulls_not_distinct_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "nulls_not_distinct_2.yml"),
        // these statements must stay bounded
        ("nulls_not_distinct_bounded", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "nulls_not_distinct_bounded.yml"),
        // interacting script kept intact
        ("joins_and_conjunctions", "test-cases/rules/postgres-bounded-statements/fixture/unique-index-shapes", "joins_and_conjunctions.yml"),
        // tables that must remain: normalized_emails
        ("nullsafe_expressions_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "nullsafe_expressions_2.yml"),
        // these statements must stay bounded
        ("nullsafe_expressions_bounded", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "nullsafe_expressions_bounded.yml"),
        // tables that must remain: nd_contacts
        ("index_guards_1", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "index_guards_1.yml"),
        // tables that must remain: normalized_emails
        ("index_guards_3", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "index_guards_3.yml"),
        // tables that must remain: partial_expr_nullguard
        ("index_guards_4", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "index_guards_4.yml"),
        // these statements must stay bounded
        ("index_guards_bounded", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "index_guards_bounded.yml"),
        // tables that must remain: broken_predicate
        ("guards_malformed_1", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "guards_malformed_1.yml"),
        // tables that must remain: broken_expression
        ("guards_malformed_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "guards_malformed_2.yml"),
        // accounts must remain a finding
        ("joined_alias_timing_1", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "joined_alias_timing_1.yml"),
        // accounts must remain a finding
        ("joined_alias_timing_3", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "joined_alias_timing_3.yml"),
        // accounts must remain a finding
        ("joined_alias_timing_5", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "joined_alias_timing_5.yml"),
        // accounts must remain a finding
        ("joined_alias_timing_6", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "joined_alias_timing_6.yml"),
        // accounts must remain a finding
        ("joined_alias_timing_7", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "joined_alias_timing_7.yml"),
        // accounts must remain a finding
        ("joined_alias_timing_9", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "joined_alias_timing_9.yml"),
        // these DELETE statements must stay bounded
        ("joined_alias_timing_bounded", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "joined_alias_timing_bounded.yml"),
        // accounts must remain a finding
        ("join_on_scope_1", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "join_on_scope_1.yml"),
        // accounts must remain a finding
        ("join_on_scope_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "join_on_scope_2.yml"),
        // accounts must remain a finding
        ("join_on_scope_3", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "join_on_scope_3.yml"),
        // accounts must remain a finding
        ("join_on_scope_4", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "join_on_scope_4.yml"),
        // accounts must remain a finding
        ("join_on_scope_5", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "join_on_scope_5.yml"),
        // accounts must remain a finding
        ("join_on_scope_6", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "join_on_scope_6.yml"),
        // accounts must remain a finding
        ("join_on_scope_7", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "join_on_scope_7.yml"),
        // these DELETE statements must stay bounded
        ("join_on_scope_bounded", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "join_on_scope_bounded.yml"),
        // tables that must remain: accounts
        ("a_whole_unique_key_pinned_to_a_value_bounds_a_relation_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_whole_unique_key_pinned_to_a_value_bounds_a_relation_2.yml"),
        // tables that must remain: accounts
        ("a_whole_unique_key_pinned_to_a_value_bounds_a_relation_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_whole_unique_key_pinned_to_a_value_bounds_a_relation_3_2.yml"),
        // tables that must remain: order_lines
        ("a_whole_unique_key_pinned_to_a_value_bounds_a_relation_4_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_whole_unique_key_pinned_to_a_value_bounds_a_relation_4_2.yml"),
        // tables that must remain: orders
        ("a_limit_a_pure_aggregate_or_an_unknown_relation_needs_no_key_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_limit_a_pure_aggregate_or_an_unknown_relation_needs_no_key_2.yml"),
        // tables that must remain: accounts
        ("an_opaque_table_function_bounds_nothing_but_a_caller_sized_one_does_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "an_opaque_table_function_bounds_nothing_but_a_caller_sized_one_does_2.yml"),
        // tables that must remain: accounts
        ("an_opaque_table_function_bounds_nothing_but_a_caller_sized_one_does_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "an_opaque_table_function_bounds_nothing_but_a_caller_sized_one_does_3_2.yml"),
        // this statement must stay bounded
        ("an_opaque_table_function_bounds_nothing_but_a_caller_sized_one_does_4_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "an_opaque_table_function_bounds_nothing_but_a_caller_sized_one_does_4_2.yml"),
        // this statement must stay bounded
        ("an_opaque_table_function_bounds_nothing_but_a_caller_sized_one_does_5_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "an_opaque_table_function_bounds_nothing_but_a_caller_sized_one_does_5_2.yml"),
        // tables that must remain: accounts
        ("a_recursive_reference_does_not_bound_the_relations_joined_to_it_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_recursive_reference_does_not_bound_the_relations_joined_to_it_2.yml"),
        // tables that must remain: orders
        ("a_count_taken_from_the_data_is_not_a_cap_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_count_taken_from_the_data_is_not_a_cap_2.yml"),
        // tables that must remain: orders
        ("a_count_taken_from_the_data_is_not_a_cap_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_count_taken_from_the_data_is_not_a_cap_3_2.yml"),
        // tables that must remain: orders
        ("a_count_taken_from_the_data_is_not_a_cap_4_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_count_taken_from_the_data_is_not_a_cap_4_2.yml"),
        // tables that must remain: orders
        ("aggregates_cap_through_having_but_not_through_an_expanding_select_list_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "aggregates_cap_through_having_but_not_through_an_expanding_select_list_2.yml"),
        // tables that must remain: orders
        ("a_query_run_by_copy_is_judged_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_query_run_by_copy_is_judged_2.yml"),
        // tables that must remain: labels
        ("a_key_with_a_non_default_operator_class_or_collation_is_not_a_key_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_key_with_a_non_default_operator_class_or_collation_is_not_a_key_2.yml"),
        // tables that must remain: accounts
        ("a_relation_in_another_schema_is_not_judged_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_relation_in_another_schema_is_not_judged_2.yml"),
        // tables that must remain: accounts
        ("a_relation_in_another_schema_is_not_judged_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_relation_in_another_schema_is_not_judged_3_2.yml"),
        // tables that must remain: accounts
        ("a_lateral_source_that_reads_the_row_bounds_nothing_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_lateral_source_that_reads_the_row_bounds_nothing_2.yml"),
        // tables that must remain: orders
        ("a_lateral_source_that_reads_the_row_bounds_nothing_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_lateral_source_that_reads_the_row_bounds_nothing_3_2.yml"),
        // tables that must remain: accounts
        ("an_unknown_relation_has_no_ctid_key_either_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "an_unknown_relation_has_no_ctid_key_either_2.yml"),
        // tables that must remain: accounts
        ("nested_aliases_do_not_hide_an_outer_row_reference_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "nested_aliases_do_not_hide_an_outer_row_reference_2.yml"),
        // tables that must remain: accounts
        ("nested_aliases_do_not_hide_an_outer_row_reference_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "nested_aliases_do_not_hide_an_outer_row_reference_3_2.yml"),
        // tables that must remain: accounts
        ("a_table_function_that_reads_another_item_or_comes_from_a_user_schema_bounds_noth_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_table_function_that_reads_another_item_or_comes_from_a_user_schema_bounds_noth_2.yml"),
        // tables that must remain: accounts
        ("a_table_function_that_reads_another_item_or_comes_from_a_user_schema_bounds_noth_4", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_table_function_that_reads_another_item_or_comes_from_a_user_schema_bounds_noth_4.yml"),
        // tables that must remain: orders
        ("only_a_non_null_function_of_fixed_inputs_fixes_a_count_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "only_a_non_null_function_of_fixed_inputs_fixes_a_count_2.yml"),
        // tables that must remain: orders
        ("set_operations_read_every_arm_so_every_arm_must_be_bounded_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "set_operations_read_every_arm_so_every_arm_must_be_bounded_2.yml"),
        // tables that must remain: orders
        ("set_operations_read_every_arm_so_every_arm_must_be_bounded_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "set_operations_read_every_arm_so_every_arm_must_be_bounded_3_2.yml"),
        // tables that must remain: accounts
        ("a_bare_column_that_no_table_of_the_subquery_has_reads_the_row_checked_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_bare_column_that_no_table_of_the_subquery_has_reads_the_row_checked_2.yml"),
        // tables that must remain: accounts
        ("a_bare_column_that_no_table_of_the_subquery_has_reads_the_row_checked_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_bare_column_that_no_table_of_the_subquery_has_reads_the_row_checked_3_2.yml"),
        // tables that must remain: accounts
        ("a_bare_column_that_no_table_of_the_subquery_has_reads_the_row_checked_4_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_bare_column_that_no_table_of_the_subquery_has_reads_the_row_checked_4_2.yml"),
        // tables that must remain: accounts
        ("a_bare_column_that_no_table_of_the_subquery_has_reads_the_row_checked_5_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_bare_column_that_no_table_of_the_subquery_has_reads_the_row_checked_5_2.yml"),
        // tables that must remain: accounts
        ("a_bare_column_that_no_table_of_the_subquery_has_reads_the_row_checked_6_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_bare_column_that_no_table_of_the_subquery_has_reads_the_row_checked_6_2.yml"),
        // tables that must remain: orders
        ("a_data_modifying_cte_bounds_joined_rows_only_when_its_target_is_bounded_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_data_modifying_cte_bounds_joined_rows_only_when_its_target_is_bounded_2.yml"),
        // tables that must remain: orders
        ("a_table_arm_that_names_a_cte_carries_the_cte_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_table_arm_that_names_a_cte_carries_the_cte_2.yml"),
        // tables that must remain: orders
        ("a_table_arm_that_names_a_cte_carries_the_cte_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_table_arm_that_names_a_cte_carries_the_cte_3_2.yml"),
        // tables that must remain: accounts
        ("an_explicit_collation_in_the_compared_value_fixes_no_row_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "an_explicit_collation_in_the_compared_value_fixes_no_row_2.yml"),
        // tables that must remain: orders
        ("jsonpath_set_returning_functions_expand_aggregate_rows_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "jsonpath_set_returning_functions_expand_aggregate_rows_2.yml"),
        // this statement must stay bounded
        ("a_recovered_interpolation_pins_like_a_placeholder_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_recovered_interpolation_pins_like_a_placeholder_2.yml"),
        // tables that must remain: invoices
        ("a_recovered_interpolation_pins_like_a_placeholder_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_recovered_interpolation_pins_like_a_placeholder_3_2.yml"),
        // tables that must remain: orders
        ("a_bound_propagates_across_joins_on_unique_keys_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_bound_propagates_across_joins_on_unique_keys_2.yml"),
        // tables that must remain: orders, order_lines
        ("a_bound_propagates_across_joins_on_unique_keys_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_bound_propagates_across_joins_on_unique_keys_3_2.yml"),
        // tables that must remain: orders, accounts
        ("bounded_items_bound_the_relations_pinned_to_them_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "bounded_items_bound_the_relations_pinned_to_them_2.yml"),
        // tables that must remain: orders, accounts
        ("bounded_items_bound_the_relations_pinned_to_them_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "bounded_items_bound_the_relations_pinned_to_them_3_2.yml"),
        // tables that must remain: orders
        ("set_operations_need_every_arm_bounded_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "set_operations_need_every_arm_bounded_2.yml"),
        // tables that must remain: orders
        ("a_derived_table_is_bounded_when_its_query_is_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_derived_table_is_bounded_when_its_query_is_2.yml"),
        // this statement must stay bounded
        ("updates_and_deletes_need_only_their_target_bounded_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "updates_and_deletes_need_only_their_target_bounded_2.yml"),
        // tables that must remain: orders
        ("updates_and_deletes_need_only_their_target_bounded_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "updates_and_deletes_need_only_their_target_bounded_3_2.yml"),
        // tables that must remain: orders
        ("updates_and_deletes_need_only_their_target_bounded_4_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "updates_and_deletes_need_only_their_target_bounded_4_2.yml"),
        // tables that must remain: orders
        ("a_relation_the_catalog_names_in_another_form_is_found_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_relation_the_catalog_names_in_another_form_is_found_2.yml"),
        // tables that must remain: orders
        ("a_relation_the_catalog_names_in_another_form_is_found_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_relation_the_catalog_names_in_another_form_is_found_3_2.yml"),
        // relation lines must match the statement text; tables that must remain: orders, accounts
        ("offenders_carry_the_line_of_each_relation_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "offenders_carry_the_line_of_each_relation_2.yml"),
        // tables that must remain: accounts
        ("an_unknown_relation_is_not_judged_and_bounds_nothing_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "an_unknown_relation_is_not_judged_and_bounds_nothing_2.yml"),
        // tables that must remain: accounts
        ("an_unknown_relation_is_not_judged_and_bounds_nothing_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "an_unknown_relation_is_not_judged_and_bounds_nothing_3_2.yml"),
        // tables that must remain: accounts
        ("an_unknown_relation_is_not_judged_and_bounds_nothing_4_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "an_unknown_relation_is_not_judged_and_bounds_nothing_4_2.yml"),
        // tables that must remain: accounts
        ("a_subquery_that_reads_the_row_is_not_a_key_pin_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_subquery_that_reads_the_row_is_not_a_key_pin_2.yml"),
        // tables that must remain: accounts
        ("a_subquery_that_reads_the_row_is_not_a_key_pin_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_subquery_that_reads_the_row_is_not_a_key_pin_3_2.yml"),
        // tables that must remain: accounts
        ("a_subquery_that_reads_the_row_is_not_a_key_pin_4_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_subquery_that_reads_the_row_is_not_a_key_pin_4_2.yml"),
        // tables that must remain: accounts
        ("a_subquery_that_reads_the_row_is_not_a_key_pin_5_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_subquery_that_reads_the_row_is_not_a_key_pin_5_2.yml"),
        // tables that must remain: accounts
        ("a_subquery_that_reads_the_row_is_not_a_key_pin_6_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_subquery_that_reads_the_row_is_not_a_key_pin_6_2.yml"),
        // tables that must remain: accounts
        ("a_function_that_differs_per_row_is_not_a_fixed_value_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_function_that_differs_per_row_is_not_a_fixed_value_2.yml"),
        // tables that must remain: accounts
        ("a_function_that_differs_per_row_is_not_a_fixed_value_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_function_that_differs_per_row_is_not_a_fixed_value_3_2.yml"),
        // tables that must remain: accounts
        ("a_function_that_differs_per_row_is_not_a_fixed_value_4_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_function_that_differs_per_row_is_not_a_fixed_value_4_2.yml"),
        // tables that must remain: accounts
        ("a_function_that_differs_per_row_is_not_a_fixed_value_5_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_function_that_differs_per_row_is_not_a_fixed_value_5_2.yml"),
        // tables that must remain: orders
        ("table_returns_every_row_of_its_relation_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "table_returns_every_row_of_its_relation_2.yml"),
        // tables that must remain: accounts
        ("table_returns_every_row_of_its_relation_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "table_returns_every_row_of_its_relation_3_2.yml"),
        // tables that must remain: accounts
        ("a_dml_target_is_a_physical_relation_even_when_a_cte_shares_its_name_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_dml_target_is_a_physical_relation_even_when_a_cte_shares_its_name_2.yml"),
        // tables that must remain: orders
        ("a_dml_target_is_a_physical_relation_even_when_a_cte_shares_its_name_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_dml_target_is_a_physical_relation_even_when_a_cte_shares_its_name_3_2.yml"),
        // this statement must stay bounded
        ("a_dml_target_is_a_physical_relation_even_when_a_cte_shares_its_name_4_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_dml_target_is_a_physical_relation_even_when_a_cte_shares_its_name_4_2.yml"),
        // tables that must remain: orders, accounts
        ("a_relation_reached_by_several_arms_is_reported_once_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_relation_reached_by_several_arms_is_reported_once_2.yml"),
        // tables that must remain: events
        ("ctid_is_not_a_key_of_a_partitioned_table_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "ctid_is_not_a_key_of_a_partitioned_table_2.yml"),
        // tables that must remain: events
        ("ctid_is_not_a_key_of_a_partitioned_table_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "ctid_is_not_a_key_of_a_partitioned_table_3_2.yml"),
        // tables that must remain: accounts, orders
        ("join_using_bounds_like_on_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "join_using_bounds_like_on_2.yml"),
        // tables that must remain: orders
        ("join_using_bounds_like_on_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "join_using_bounds_like_on_3_2.yml"),
        // this statement must report a finding
        ("unique_keys_resolve_through_the_same_identity_as_the_table_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "unique_keys_resolve_through_the_same_identity_as_the_table_2.yml"),
        // tables that must remain: contacts
        ("a_null_safe_comparison_bounds_only_a_not_null_key_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "a_null_safe_comparison_bounds_only_a_not_null_key_2.yml"),
        // tables that must remain: accounts
        ("outer_joins_bound_only_the_side_they_cannot_null_extend_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "outer_joins_bound_only_the_side_they_cannot_null_extend_2.yml"),
        // tables that must remain: accounts
        ("outer_joins_bound_only_the_side_they_cannot_null_extend_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "outer_joins_bound_only_the_side_they_cannot_null_extend_3_2.yml"),
        // tables that must remain: orders
        ("fetch_with_ties_and_foreign_aggregates_do_not_cap_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "fetch_with_ties_and_foreign_aggregates_do_not_cap_2.yml"),
        // tables that must remain: orders
        ("fetch_with_ties_and_foreign_aggregates_do_not_cap_3_2", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "fetch_with_ties_and_foreign_aggregates_do_not_cap_3_2.yml"),
        // every statement in this file must stay bounded
        ("bounded_schema", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "bounded_schema.yml"),
        // every statement in this file must stay bounded
        ("bounded_catalogs_schema_qualified", "test-cases/rules/postgres-bounded-statements/snapshot-inline", "bounded_catalogs_schema_qualified.yml"),
        ("referential_actions_do_not_expand_the_direct_statement_scope_referential_actions", "test-cases/rules/postgres-bounded-statements/fixture", "referential_actions_do_not_expand_the_direct_statement_scope_referential_actions.yml"),
        ("bounded_query_outputs_preserve_nested_pin_read_offenders_nested_pin_offenders", "test-cases/rules/postgres-bounded-statements/fixture", "bounded_query_outputs_preserve_nested_pin_read_offenders_nested_pin_offenders.yml"),
        ("scalar_reducers_keep_argument_row_bounds_without_scalar_argument_types_scalar_re", "test-cases/rules/postgres-bounded-statements/fixture", "scalar_reducers_keep_argument_row_bounds_without_scalar_argument_types_scalar_re.yml"),
        ("pin_query_reads_survive_oversized_cte_compaction_compacted_pin_query_reads", "test-cases/rules/postgres-bounded-statements/fixture", "pin_query_reads_survive_oversized_cte_compaction_compacted_pin_query_reads.yml"),
        ("correlated_pin_dependencies_retire_temporary_views_after_cascade_temp_view_pin_d", "test-cases/rules/postgres-bounded-statements/fixture", "correlated_pin_dependencies_retire_temporary_views_after_cascade_temp_view_pin_d.yml"),
        // lifecycle.sql must stay bounded
        ("metadata_compatibility_preserves_temporary_relations_and_prepared_plans_lifecycl", "fixtures/postgres-facts/source/ddl-compat", "metadata_compatibility_preserves_temporary_relations_and_prepared_plans_lifecycl.yml"),
        // lifecycle-control.sql must stay non-empty
        ("metadata_compatibility_preserves_temporary_relations_and_prepared_plans_lifecycl_2", "fixtures/postgres-facts/source/ddl-compat", "metadata_compatibility_preserves_temporary_relations_and_prepared_plans_lifecycl_2.yml"),
        ("compacted_caller_arrays_retain_type_proofs_without_relation_dependencies_oversiz", "test-cases/rules/postgres-bounded-statements/fixture", "compacted_caller_arrays_retain_type_proofs_without_relation_dependencies_oversiz.yml"),
        ("table_functions_do_not_hide_outer_schema_qualified_table_references_function_cor", "test-cases/rules/postgres-bounded-statements/fixture", "table_functions_do_not_hide_outer_schema_qualified_table_references_function_cor.yml"),
        ("quoted_table_arms_respect_temporary_identity_and_view_dependencies_temporary_tab", "test-cases/rules/postgres-bounded-statements/fixture", "quoted_table_arms_respect_temporary_identity_and_view_dependencies_temporary_tab.yml"),
        ("quoted_table_arm_in_a_strictly_parsed_source_shadows_the_catalog_temporary_table", "test-cases/rules/postgres-bounded-statements/fixture", "quoted_table_arm_in_a_strictly_parsed_source_shadows_the_catalog_temporary_table.yml"),
        ("repeated_quoted_table_arms_do_not_consume_tokens_twice_temporary_table_quoted_re", "test-cases/rules/postgres-bounded-statements/fixture", "repeated_quoted_table_arms_do_not_consume_tokens_twice_temporary_table_quoted_re.yml"),
        ("skipped_create_table_arm_cannot_supply_a_later_query_identity_temporary_table_qu", "test-cases/rules/postgres-bounded-statements/fixture", "skipped_create_table_arm_cannot_supply_a_later_query_identity_temporary_table_qu.yml"),
        ("ignored_scalar_table_arm_cannot_supply_an_outer_arm_identity_temporary_table_quo", "test-cases/rules/postgres-bounded-statements/fixture", "ignored_scalar_table_arm_cannot_supply_an_outer_arm_identity_temporary_table_quo.yml"),
        ("ignored_scalar_table_arm_does_not_steal_a_quoted_outer_identity_temporary_table_", "test-cases/rules/postgres-bounded-statements/fixture", "ignored_scalar_table_arm_does_not_steal_a_quoted_outer_identity_temporary_table_.yml"),
        ("ignored_projection_table_arm_cannot_mask_a_derived_relation_temporary_table_quot", "test-cases/rules/postgres-bounded-statements/fixture", "ignored_projection_table_arm_cannot_mask_a_derived_relation_temporary_table_quot.yml"),
        ("ignored_projection_table_arm_cannot_rename_a_direct_derived_relation_table_quote", "test-cases/rules/postgres-bounded-statements/fixture", "ignored_projection_table_arm_cannot_rename_a_direct_derived_relation_table_quote.yml"),
        ("materialized_drops_preserve_temporary_wrong_kind_namesakes_temporary_materialize", "test-cases/rules/postgres-bounded-statements/fixture", "materialized_drops_preserve_temporary_wrong_kind_namesakes_temporary_materialize.yml"),
        ("physical_schema_renames_restore_catalog_findings_temporary_schema_rename", "test-cases/rules/postgres-bounded-statements/fixture", "physical_schema_renames_restore_catalog_findings_temporary_schema_rename.yml"),
        ("sole_legacy_columns_map_aliases_without_claiming_ambiguous_or_system_columns_leg", "test-cases/rules/postgres-bounded-statements/fixture", "sole_legacy_columns_map_aliases_without_claiming_ambiguous_or_system_columns_leg.yml"),
        ("table_only_arms_keep_temporary_and_permanent_names_separate_temporary_table_only", "test-cases/rules/postgres-bounded-statements/fixture", "table_only_arms_keep_temporary_and_permanent_names_separate_temporary_table_only.yml"),
        ("a_strict_table_only_arm_shadows_the_permanent_catalog_temporary_table_only_singl", "test-cases/rules/postgres-bounded-statements/fixture", "a_strict_table_only_arm_shadows_the_permanent_catalog_temporary_table_only_singl.yml"),
        ("prepared_select_into_shadows_only_after_execute_temporary_prepared_into", "test-cases/rules/postgres-bounded-statements/fixture", "prepared_select_into_shadows_only_after_execute_temporary_prepared_into.yml"),
        ("rejected_prepared_transitions_keep_catalog_reads_visible_temporary_prepared_vali", "test-cases/rules/postgres-bounded-statements/fixture", "rejected_prepared_transitions_keep_catalog_reads_visible_temporary_prepared_vali.yml"),
        ("partial_prepared_types_keep_wrong_arity_catalog_reads_visible_temporary_prepared", "test-cases/rules/postgres-bounded-statements/fixture", "partial_prepared_types_keep_wrong_arity_catalog_reads_visible_temporary_prepared.yml"),
        ("prepared_name_truncation_limits_catalog_reads_to_nonmatching_executions_temporar", "test-cases/rules/postgres-bounded-statements/fixture", "prepared_name_truncation_limits_catalog_reads_to_nonmatching_executions_temporar.yml"),
        ("outer_caps_preserve_only_blocking_input_reads_blocking_query_wrappers", "test-cases/rules/postgres-bounded-statements/fixture", "outer_caps_preserve_only_blocking_input_reads_blocking_query_wrappers.yml"),
        ("compacted_wrappers_keep_blocking_reads_but_zero_executes_no_inputs_compacted_blo", "test-cases/rules/postgres-bounded-statements/fixture", "compacted_wrappers_keep_blocking_reads_but_zero_executes_no_inputs_compacted_blo.yml"),
        // every statement must stay bounded
        ("returning_select_pk", "test-cases/rules/postgres-bounded-statements/fixture", "returning_select_pk.yml"),
        // every statement must stay bounded
        ("returning_update_pk", "test-cases/rules/postgres-bounded-statements/fixture", "returning_update_pk.yml"),
        // every statement must stay bounded
        ("returning_delete_pk", "test-cases/rules/postgres-bounded-statements/fixture", "returning_delete_pk.yml"),
        // every statement must stay bounded
        ("returning_nested_projection", "test-cases/rules/postgres-bounded-statements/fixture", "returning_nested_projection.yml"),
        // every statement must stay bounded
        ("returning_nested_with_bounded", "test-cases/rules/postgres-bounded-statements/fixture", "returning_nested_with_bounded.yml"),
        // every statement must stay bounded
        ("returning_auxiliary_independent", "test-cases/rules/postgres-bounded-statements/fixture", "returning_auxiliary_independent.yml"),
        // every statement must stay bounded
        ("returning_no_returning_unused", "test-cases/rules/postgres-bounded-statements/fixture", "returning_no_returning_unused.yml"),
        // tables that must remain: orders, accounts
        ("returning_unbounded_update", "test-cases/rules/postgres-bounded-statements/fixture", "returning_unbounded_update.yml"),
        // tables that must remain: orders, accounts
        ("returning_auxiliary_unbounded", "test-cases/rules/postgres-bounded-statements/fixture", "returning_auxiliary_unbounded.yml"),
        // tables that must remain: orders, accounts
        ("returning_nested_with_namesake", "test-cases/rules/postgres-bounded-statements/fixture", "returning_nested_with_namesake.yml"),
        // accounts must remain; the other statement must stay bounded
        ("returning_non_pk_join", "test-cases/rules/postgres-bounded-statements/fixture", "returning_non_pk_join.yml"),
        // accounts must remain; the other statement must stay bounded
        ("returning_recursive_opaque", "test-cases/rules/postgres-bounded-statements/fixture", "returning_recursive_opaque.yml"),
        // accounts must remain; two statements must stay bounded
        ("returning_declaration_time_scope", "test-cases/rules/postgres-bounded-statements/fixture", "returning_declaration_time_scope.yml"),
        ("nonrecursive_cte_base_reads_do_not_invent_outer_row_dependence_nonrecursive_cte_", "test-cases/rules/postgres-bounded-statements/fixture", "nonrecursive_cte_base_reads_do_not_invent_outer_row_dependence_nonrecursive_cte_.yml"),
        ("streaming_table_arm_with_derived_limit_is_bounded_table_derived_streaming_limit", "test-cases/rules/postgres-bounded-statements/fixture", "streaming_table_arm_with_derived_limit_is_bounded_table_derived_streaming_limit.yml"),
        ("blocking_union_reads_table_before_derived_limit_table_derived_blocking_limit", "test-cases/rules/postgres-bounded-statements/fixture", "blocking_union_reads_table_before_derived_limit_table_derived_blocking_limit.yml"),
        ("rewritten_derived_table_arms_keep_quoted_identity_table_derived_quoted_control", "test-cases/rules/postgres-bounded-statements/fixture", "rewritten_derived_table_arms_keep_quoted_identity_table_derived_quoted_control.yml"),
        ("aliased_nested_joins_preserve_catalog_column_ownership_aliased_nested_joins", "test-cases/rules/postgres-bounded-statements/fixture", "aliased_nested_joins_preserve_catalog_column_ownership_aliased_nested_joins.yml"),
        ("nested_zero_caps_survive_outer_order_expansion_nested_zero_order", "test-cases/rules/postgres-bounded-statements/fixture", "nested_zero_caps_survive_outer_order_expansion_nested_zero_order.yml"),
        ("catalog_trigger_fanout_stays_outside_direct_target_row_bounds_referential_action", "test-cases/rules/postgres-bounded-statements/fixture", "catalog_trigger_fanout_stays_outside_direct_target_row_bounds_referential_action.yml"),
        ("qualified_relations_keep_their_schema_in_correlation_checks_schema_correlation", "test-cases/rules/postgres-bounded-statements/fixture", "qualified_relations_keep_their_schema_in_correlation_checks_schema_correlation.yml"),
        ("partial_derived_aliases_keep_the_projection_suffix_local_partial_projection_alia", "test-cases/rules/postgres-bounded-statements/fixture", "partial_derived_aliases_keep_the_projection_suffix_local_partial_projection_alia.yml"),
        ("partial_derived_aliases_keep_the_projection_suffix_local_partial_alias_outer_cor", "test-cases/rules/postgres-bounded-statements/fixture", "partial_derived_aliases_keep_the_projection_suffix_local_partial_alias_outer_cor.yml"),
        ("a_qualified_reference_to_the_same_catalog_relation_is_local_qualified_catalog_lo", "test-cases/rules/postgres-bounded-statements/fixture", "a_qualified_reference_to_the_same_catalog_relation_is_local_qualified_catalog_lo.yml"),
        ("an_unknown_inner_relation_keeps_qualified_reads_conservative_qualified_catalog_u", "test-cases/rules/postgres-bounded-statements/fixture", "an_unknown_inner_relation_keeps_qualified_reads_conservative_qualified_catalog_u.yml"),
        ("an_explicit_inner_alias_does_not_make_a_qualified_outer_read_local_qualified_cat", "test-cases/rules/postgres-bounded-statements/fixture", "an_explicit_inner_alias_does_not_make_a_qualified_outer_read_local_qualified_cat.yml"),
        ("a_join_alias_hides_its_children_from_catalog_qualified_locality_qualified_catalo", "test-cases/rules/postgres-bounded-statements/fixture", "a_join_alias_hides_its_children_from_catalog_qualified_locality_qualified_catalo.yml"),
        ("hidden_join_two_part_qualifier_keeps_outer_write_unbounded_hidden_join_two_part", "test-cases/rules/postgres-bounded-statements/fixture", "hidden_join_two_part_qualifier_keeps_outer_write_unbounded_hidden_join_two_part.yml"),
        ("a_later_join_child_cannot_shadow_an_earlier_on_outer_read_join_forward_alias", "test-cases/rules/postgres-bounded-statements/fixture", "a_later_join_child_cannot_shadow_an_earlier_on_outer_read_join_forward_alias.yml"),
        ("set_operation_arms_do_not_share_relation_aliases_set_arm_alias_scope", "test-cases/rules/postgres-bounded-statements/fixture", "set_operation_arms_do_not_share_relation_aliases_set_arm_alias_scope.yml"),
        ("group_only_names_remain_correlated_unless_the_arm_projects_them_set_arm_group_on", "test-cases/rules/postgres-bounded-statements/fixture", "group_only_names_remain_correlated_unless_the_arm_projects_them_set_arm_group_on.yml"),
        ("retained_stored_array_pins_never_credit_a_bounded_owner_stored_array_facts", "test-cases/rules/postgres-bounded-statements/fixture", "retained_stored_array_pins_never_credit_a_bounded_owner_stored_array_facts.yml"),
        ("wrapped_self_arrays_remain_unbounded_wrapped_stored_array_facts", "test-cases/rules/postgres-bounded-statements/fixture", "wrapped_self_arrays_remain_unbounded_wrapped_stored_array_facts.yml"),
        ("standalone_table_queries_keep_temporary_identity_and_report_the_catalog_namesake", "test-cases/rules/postgres-bounded-statements/fixture", "standalone_table_queries_keep_temporary_identity_and_report_the_catalog_namesake.yml"),
        ("lenient_recovery_keeps_standalone_table_queries_after_unsupported_ddl_standalone", "test-cases/rules/postgres-bounded-statements/fixture", "lenient_recovery_keeps_standalone_table_queries_after_unsupported_ddl_standalone.yml"),
        ("qualified_and_quoted_conditional_function_lookalikes_are_not_caps_qualified_limi", "test-cases/rules/postgres-bounded-statements/fixture", "qualified_and_quoted_conditional_function_lookalikes_are_not_caps_qualified_limi.yml"),
        ("data_backed_select_list_srfs_do_not_bound_joined_targets_select_list_srf", "test-cases/rules/postgres-bounded-statements/fixture", "data_backed_select_list_srfs_do_not_bound_joined_targets_select_list_srf.yml"),
        ("permanent_intermediate_view_cascade_reveals_catalog_relations_temporary_permanen", "test-cases/rules/postgres-bounded-statements/fixture", "permanent_intermediate_view_cascade_reveals_catalog_relations_temporary_permanen.yml"),
        ("physical_view_source_rename_reveals_temporary_dependent_after_cascade_temporary_", "test-cases/rules/postgres-bounded-statements/fixture", "physical_view_source_rename_reveals_temporary_dependent_after_cascade_temporary_.yml"),
        ("schema_rename_moves_permanent_intermediary_identity_before_cascade_temporary_per", "test-cases/rules/postgres-bounded-statements/fixture", "schema_rename_moves_permanent_intermediary_identity_before_cascade_temporary_per.yml"),
        ("ambiguous_source_rename_preserves_later_catalog_finding_temporary_permanent_view", "test-cases/rules/postgres-bounded-statements/fixture", "ambiguous_source_rename_preserves_later_catalog_finding_temporary_permanent_view.yml"),
        ("permanent_view_node_rename_preserves_cascade_identity_and_failed_ddl_temporary_p", "test-cases/rules/postgres-bounded-statements/fixture", "permanent_view_node_rename_preserves_cascade_identity_and_failed_ddl_temporary_p.yml"),
        ("bare_view_collision_preserves_each_schema_candidate_temporary_permanent_view_bar", "test-cases/rules/postgres-bounded-statements/fixture", "bare_view_collision_preserves_each_schema_candidate_temporary_permanent_view_bar.yml"),
        ("recursive_forward_ctes_report_the_unbounded_catalog_leaf_recursive_forward_bound", "test-cases/rules/postgres-bounded-statements/fixture", "recursive_forward_ctes_report_the_unbounded_catalog_leaf_recursive_forward_bound.yml"),
        ("recursive_table_arms_report_exact_catalog_leaves_recursive_table_forward", "test-cases/rules/postgres-bounded-statements/fixture", "recursive_table_arms_report_exact_catalog_leaves_recursive_table_forward.yml"),
        ("recursive_table_arms_report_exact_catalog_leaves_recursive_table_quoted_union", "test-cases/rules/postgres-bounded-statements/fixture", "recursive_table_arms_report_exact_catalog_leaves_recursive_table_quoted_union.yml"),
        ("recursive_table_arms_report_exact_catalog_leaves_recursive_table_qualified_union", "test-cases/rules/postgres-bounded-statements/fixture", "recursive_table_arms_report_exact_catalog_leaves_recursive_table_qualified_union.yml"),
        ("proven_physical_shadow_drop_exposes_the_underlying_temporary_rows_temporary_shad", "test-cases/rules/postgres-bounded-statements/fixture", "proven_physical_shadow_drop_exposes_the_underlying_temporary_rows_temporary_shad.yml"),
        ("unknown_shadow_drop_never_lends_catalog_keys_to_the_join_temporary_shadow_drop_u", "test-cases/rules/postgres-bounded-statements/fixture", "unknown_shadow_drop_never_lends_catalog_keys_to_the_join_temporary_shadow_drop_u.yml"),
        ("database_qualified_shadow_drop_requires_a_matching_current_database_schema_searc", "test-cases/rules/postgres-bounded-statements/fixture", "database_qualified_shadow_drop_requires_a_matching_current_database_schema_searc.yml"),
        ("database_qualified_shadow_drop_requires_a_matching_current_database_schema_searc_2", "test-cases/rules/postgres-bounded-statements/fixture", "database_qualified_shadow_drop_requires_a_matching_current_database_schema_searc_2.yml"),
        ("recreated_physical_shadows_never_retire_the_underlying_temp_relation_temporary_s", "test-cases/rules/postgres-bounded-statements/fixture", "recreated_physical_shadows_never_retire_the_underlying_temp_relation_temporary_s.yml"),
        ("recreated_physical_shadows_never_retire_the_underlying_temp_relation_temporary_s_2", "test-cases/rules/postgres-bounded-statements/fixture", "recreated_physical_shadows_never_retire_the_underlying_temp_relation_temporary_s_2.yml"),
        ("recreated_physical_shadows_never_retire_the_underlying_temp_relation_temporary_s_3", "test-cases/rules/postgres-bounded-statements/fixture", "recreated_physical_shadows_never_retire_the_underlying_temp_relation_temporary_s_3.yml"),
        ("oversized_aliased_ctes_keep_the_nonunique_read_unbounded_oversized_cte_aliased_p", "test-cases/rules/postgres-bounded-statements/fixture", "oversized_aliased_ctes_keep_the_nonunique_read_unbounded_oversized_cte_aliased_p.yml"),
        ("three_arms_keep_temporary_and_permanent_namesakes_separate_multi_table_arms", "test-cases/rules/postgres-bounded-statements/fixture", "three_arms_keep_temporary_and_permanent_namesakes_separate_multi_table_arms.yml"),
        ("three_arms_keep_temporary_and_permanent_namesakes_separate_multi_table_arms_leni", "test-cases/rules/postgres-bounded-statements/fixture", "three_arms_keep_temporary_and_permanent_namesakes_separate_multi_table_arms_leni.yml"),
        ("fetch_expression_counts_preserve_bound_and_ties_policy_fetch_expressions", "test-cases/rules/postgres-bounded-statements/fixture", "fetch_expression_counts_preserve_bound_and_ties_policy_fetch_expressions.yml"),
        ("unknown_function_inputs_do_not_bound_table_functions_function_inputs", "test-cases/rules/postgres-bounded-statements/fixture", "unknown_function_inputs_do_not_bound_table_functions_function_inputs.yml"),
        ("temporary_view_dependencies_follow_physical_cascade_temporary_physical_cascade", "test-cases/rules/postgres-bounded-statements/fixture", "temporary_view_dependencies_follow_physical_cascade_temporary_physical_cascade.yml"),
        ("temporary_shadow_cannot_borrow_permanent_bare_column_ownership_temporary_bare_ow", "test-cases/rules/postgres-bounded-statements/fixture", "temporary_shadow_cannot_borrow_permanent_bare_column_ownership_temporary_bare_ow.yml"),
        ("scalar_case_boundaries_preserve_row_ownership_and_standalone_unknown_calls_scala", "test-cases/rules/postgres-bounded-statements/fixture", "scalar_case_boundaries_preserve_row_ownership_and_standalone_unknown_calls_scala.yml"),
        ("later_builtin_names_need_catalog_identity_without_a_target_version_versioned_sca", "test-cases/rules/postgres-bounded-statements/fixture", "later_builtin_names_need_catalog_identity_without_a_target_version_versioned_sca.yml"),
        ("an_explicit_self_alias_does_not_capture_a_qualified_outer_relation_explicit_self", "test-cases/rules/postgres-bounded-statements/fixture", "an_explicit_self_alias_does_not_capture_a_qualified_outer_relation_explicit_self.yml"),
        ("physical_schema_cascades_restore_findings_and_rollback_temporary_schema_drop", "test-cases/rules/postgres-bounded-statements/fixture", "physical_schema_cascades_restore_findings_and_rollback_temporary_schema_drop.yml"),
        ("nested_ctes_restore_parent_relation_visibility_nested_cte_scope", "test-cases/rules/postgres-bounded-statements/fixture", "nested_ctes_restore_parent_relation_visibility_nested_cte_scope.yml"),
        // unbounded statement indexes 0 4 6 7 9 10 must remain findings
        ("ddl_search_temporary_ddl_search_path", "test-cases/rules/postgres-bounded-statements/fixture", "ddl_search_temporary_ddl_search_path.yml"),
        // unbounded statement indexes 1 and 3 must remain findings
        ("ddl_search_temporary_ddl_search_path_local", "test-cases/rules/postgres-bounded-statements/fixture", "ddl_search_temporary_ddl_search_path_local.yml"),
        // unbounded statement indexes 2 and 3 must remain findings
        ("ddl_search_temporary_ddl_search_path_chain", "test-cases/rules/postgres-bounded-statements/fixture", "ddl_search_temporary_ddl_search_path_chain.yml"),
        ("quoted_dots_remain_inside_one_qualifier_component_quoted_qualifier_paths", "test-cases/rules/postgres-bounded-statements/fixture", "quoted_dots_remain_inside_one_qualifier_component_quoted_qualifier_paths.yml"),
        ("uncapped_selects_report_their_unbounded_pin_subqueries_pin_subquery_offenders", "test-cases/rules/postgres-bounded-statements/fixture", "uncapped_selects_report_their_unbounded_pin_subqueries_pin_subquery_offenders.yml"),
        ("declared_json_output_names_preserve_caller_key_pins_json_function_key_pins", "test-cases/rules/postgres-bounded-statements/fixture", "declared_json_output_names_preserve_caller_key_pins_json_function_key_pins.yml"),
        ("temporary_partitions_only_expose_permanent_names_after_cascade_temporary_partiti", "test-cases/rules/postgres-bounded-statements/fixture", "temporary_partitions_only_expose_permanent_names_after_cascade_temporary_partiti.yml"),
        ("partition_ddl_uses_catalog_evidence_for_the_parent_and_child_temporary_partition", "test-cases/rules/postgres-bounded-statements/fixture", "partition_ddl_uses_catalog_evidence_for_the_parent_and_child_temporary_partition.yml"),
        ("partition_ddl_uses_catalog_evidence_for_the_parent_and_child_temporary_partition_2", "test-cases/rules/postgres-bounded-statements/fixture", "partition_ddl_uses_catalog_evidence_for_the_parent_and_child_temporary_partition_2.yml"),
        ("partition_ddl_uses_catalog_evidence_for_the_parent_and_child_temporary_partition_3", "test-cases/rules/postgres-bounded-statements/fixture", "partition_ddl_uses_catalog_evidence_for_the_parent_and_child_temporary_partition_3.yml"),
        ("partition_ddl_uses_catalog_evidence_for_the_parent_and_child_temporary_partition_4", "test-cases/rules/postgres-bounded-statements/fixture", "partition_ddl_uses_catalog_evidence_for_the_parent_and_child_temporary_partition_4.yml"),
        ("oversized_ctes_retain_uncapped_relation_reads_oversized_cte", "test-cases/rules/postgres-bounded-statements/fixture", "oversized_ctes_retain_uncapped_relation_reads_oversized_cte.yml"),
    ]
    .map(|(label, root, config)| {
        (
            label,
            repo_rel(root),
            rules_path("postgres-bounded-statements", "configs", config),
        )
    });
    assert_saved_fixture("postgres_bounded_statements", &cases);
}
