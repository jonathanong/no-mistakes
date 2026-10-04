use super::names;

#[test]
fn an_opaque_table_function_bounds_nothing_but_a_caller_sized_one_does() {
    // Nothing says how many accounts the function returns.
    assert_eq!(
        names("UPDATE accounts a SET name = 'x' FROM get_all_accounts() f WHERE a.id = f.id"),
        ["accounts"]
    );
    assert_eq!(
        names("DELETE FROM accounts WHERE id = ANY(ARRAY(SELECT id FROM accounts))"),
        ["accounts"]
    );
    // The caller sizes unnest($1), a VALUES list and an array of binds.
    for sql in [
        "UPDATE accounts a SET name = 'x' FROM unnest($1::uuid[]) AS ids(id) WHERE a.id = ids.id",
        "UPDATE accounts a SET name = 'x' FROM (VALUES ($1), ($2)) AS v(id) WHERE a.id = v.id",
        "DELETE FROM accounts WHERE id = ANY(ARRAY[$1, $2])",
    ] {
        assert!(names(sql).is_empty(), "{sql}");
    }
}

#[test]
fn a_recursive_reference_does_not_bound_the_relations_joined_to_it() {
    assert_eq!(
        names(
            "WITH RECURSIVE r AS (SELECT id FROM accounts WHERE id = $1 \
             UNION ALL SELECT a.id FROM accounts a JOIN r ON a.id = r.id) SELECT 1 FROM r"
        ),
        ["accounts"]
    );
    // The outer cap bounds the recursion.
    assert!(names(
        "WITH RECURSIVE r AS (SELECT id FROM accounts WHERE id = $1 \
         UNION ALL SELECT a.id FROM accounts a JOIN r ON a.id = r.id) SELECT 1 FROM r LIMIT 10"
    )
    .is_empty());
}

#[test]
fn a_count_taken_from_the_data_is_not_a_cap() {
    for sql in [
        "SELECT * FROM orders LIMIT (SELECT count(*) FROM orders)",
        "SELECT * FROM orders LIMIT (SELECT NULL::bigint)",
        "SELECT * FROM orders LIMIT NULL::bigint",
    ] {
        assert_eq!(names(sql), ["orders"], "{sql}");
    }
    for sql in [
        "SELECT * FROM orders LIMIT $1::int",
        "SELECT * FROM orders LIMIT LEAST($1, 100)",
        "SELECT * FROM orders LIMIT sql_placeholder_1",
    ] {
        assert!(names(sql).is_empty(), "{sql}");
    }
}

#[test]
fn aggregates_cap_through_having_but_not_through_an_expanding_select_list() {
    assert!(names("SELECT 1 FROM orders HAVING count(*) > 0").is_empty());
    assert!(names("SELECT corr(1, 2), regr_slope(1, 2) FROM orders").is_empty());
    assert_eq!(
        names("SELECT generate_series(1, count(*)) FROM orders"),
        ["orders"]
    );
}

#[test]
fn a_query_run_by_copy_is_judged() {
    assert_eq!(names("COPY (SELECT * FROM orders) TO STDOUT"), ["orders"]);
    assert!(names("COPY (SELECT * FROM orders LIMIT 5) TO STDOUT").is_empty());
}

#[test]
fn a_key_with_a_non_default_operator_class_or_collation_is_not_a_key() {
    // labels.name is unique under a collation other than the column's.
    assert_eq!(names("SELECT 1 FROM labels WHERE name = $1"), ["labels"]);
    assert!(names("SELECT 1 FROM accounts WHERE email = $1").is_empty());
}

#[test]
fn a_relation_in_another_schema_is_not_judged() {
    assert!(names("SELECT * FROM audit.accounts").is_empty());
    assert_eq!(names("SELECT * FROM public.accounts"), ["accounts"]);
    assert_eq!(names("SELECT * FROM accounts"), ["accounts"]);
}

#[test]
fn a_quoted_relation_is_found_by_its_exact_name() {
    let key = "\"Order Items\"";
    assert_eq!(names("SELECT * FROM \"Order Items\""), [key]);
    assert_eq!(names("SELECT * FROM public.\"Order Items\""), [key]);
    assert!(names("SELECT * FROM \"Order Items\" WHERE id = $1").is_empty());
    assert!(names("SELECT * FROM \"Order Items\" o WHERE o.id = $1").is_empty());
    // Unquoted, the name folds to lower case: a different relation.
    assert!(names("SELECT * FROM order_items").is_empty());
}

#[test]
fn a_lateral_source_that_reads_the_row_bounds_nothing() {
    assert_eq!(
        names("SELECT * FROM accounts a JOIN LATERAL (SELECT a.id AS id) d ON a.id = d.id"),
        ["accounts"]
    );
    // The relations inside a lateral source are still judged; a capped one is bounded.
    assert_eq!(
        names(
            "SELECT 1 FROM accounts a, LATERAL (SELECT o.id FROM orders o \
             WHERE o.account_id = a.id) x WHERE a.id = $1"
        ),
        ["orders"]
    );
    assert!(names(
        "SELECT 1 FROM accounts a, LATERAL (SELECT o.id FROM orders o \
         WHERE o.account_id = a.id LIMIT 1) x WHERE a.id = $1"
    )
    .is_empty());
}

#[test]
fn an_unknown_relation_has_no_ctid_key_either() {
    // Nothing says the unknown relation is a plain table, and ctid repeats across partitions.
    assert_eq!(
        names(
            "UPDATE accounts a SET name = 'x' FROM external_events e \
             WHERE e.ctid = $1 AND a.id = e.account_id"
        ),
        ["accounts"]
    );
    assert!(names("DELETE FROM sessions WHERE ctid = $1").is_empty());
}

#[test]
fn nested_aliases_do_not_hide_an_outer_row_reference() {
    for sql in [
        "DELETE FROM accounts a WHERE a.id IN (SELECT a.id WHERE EXISTS (SELECT 1 FROM orders a LIMIT 1))",
        "DELETE FROM accounts WHERE accounts.id IN (SELECT accounts.id FROM orders WHERE orders.id IN (SELECT id FROM accounts))",
    ] {
        assert_eq!(names(sql), ["accounts"], "{sql}");
    }
}

#[test]
fn a_table_function_that_reads_another_item_or_comes_from_a_user_schema_bounds_nothing() {
    for sql in [
        "SELECT * FROM accounts a JOIN unnest(ARRAY[a.id]) u(id) ON a.id = u.id",
        "UPDATE accounts a SET name = 'x' FROM app.generate_series(1, 10) g(id) WHERE a.id = g.id",
    ] {
        assert_eq!(names(sql), ["accounts"], "{sql}");
    }
    assert!(names(
        "UPDATE accounts a SET name = 'x' FROM unnest($1::uuid[]) AS ids(id) WHERE a.id = ids.id"
    )
    .is_empty());
}

#[test]
fn only_a_non_null_function_of_fixed_inputs_fixes_a_count() {
    assert_eq!(names("SELECT * FROM orders LIMIT NULLIF(1, 1)"), ["orders"]);
    assert!(names("SELECT * FROM orders LIMIT GREATEST($1, 5)").is_empty());
}

#[test]
fn set_operations_read_every_arm_so_every_arm_must_be_bounded() {
    // The rule bounds the work, not only the result: EXCEPT and INTERSECT read both arms in full.
    assert_eq!(
        names("SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders"),
        ["orders"]
    );
    assert_eq!(
        names("SELECT id FROM accounts WHERE id = $1 INTERSECT SELECT account_id FROM orders"),
        ["orders"]
    );
}

#[test]
fn an_aggregate_only_in_order_by_bounds_the_query() {
    assert!(names("SELECT 1 FROM orders ORDER BY count(*)").is_empty());
}

#[test]
fn a_bare_column_that_no_table_of_the_subquery_has_reads_the_row_checked() {
    // `currencies` has only `code`, so PostgreSQL resolves `id` to the account's own column and
    // every account matches whenever the subquery returns a row.
    for sql in [
        "DELETE FROM accounts a WHERE a.id IN (SELECT id FROM currencies LIMIT 1)",
        "DELETE FROM accounts WHERE id = ANY (SELECT id FROM currencies LIMIT 1)",
        "DELETE FROM accounts WHERE id = (SELECT id FROM currencies LIMIT 1)",
        "UPDATE accounts SET name = $1 WHERE id IN ($2, (SELECT id FROM currencies LIMIT 1))",
        "DELETE FROM accounts WHERE id IN (SELECT code FROM currencies WHERE id = $1 LIMIT 1)",
    ] {
        assert_eq!(names(sql), ["accounts"], "{sql}");
    }
    // A table that has the column owns it, however deep the level, and so does one the catalog
    // does not describe, or a column PostgreSQL gives every table.
    for sql in [
        "DELETE FROM accounts a WHERE a.id IN (SELECT id FROM orders LIMIT 1)",
        "DELETE FROM accounts WHERE id IN (SELECT id FROM currencies c, orders LIMIT 1)",
        "DELETE FROM accounts WHERE id IN (SELECT code FROM currencies \
         WHERE EXISTS (SELECT 1 FROM orders WHERE id = $1) LIMIT 1)",
        "DELETE FROM accounts WHERE id IN (SELECT id FROM mystery_table LIMIT 1)",
        "DELETE FROM sessions WHERE ctid IN (SELECT ctid FROM currencies LIMIT 1)",
    ] {
        assert!(names(sql).is_empty(), "{sql}");
    }
}

#[test]
fn a_lateral_source_that_reads_the_row_through_a_bare_column_bounds_nothing() {
    let sql = |column: &str| {
        format!(
            "SELECT 1 FROM accounts a, LATERAL (SELECT {column} FROM currencies LIMIT 1) c, \
             orders o WHERE a.id = $1 AND o.id = c.id"
        )
    };
    // The same as naming the account's column.
    assert_eq!(names(&sql("a.id AS id")), ["orders"]);
    assert_eq!(names(&sql("id")), ["orders"]);
    // `code` is the source's own column, so it is bounded by its limit.
    assert!(names(&sql("code AS id")).is_empty());
}

#[test]
fn a_data_modifying_cte_bounds_nothing_pinned_to_it() {
    // RETURNING yields a row per modified row; nothing in the text sizes it.
    for sql in [
        "WITH moved AS (INSERT INTO archive SELECT * FROM orders RETURNING id) \
         DELETE FROM orders USING moved WHERE orders.id = moved.id",
        "WITH d AS (DELETE FROM sessions WHERE id = $1 RETURNING id) \
         SELECT 1 FROM orders o JOIN d ON o.id = d.id",
    ] {
        assert_eq!(names(sql), ["orders"], "{sql}");
    }
}

#[test]
fn a_table_arm_that_names_a_cte_carries_the_cte() {
    assert_eq!(
        names("WITH c AS (SELECT * FROM orders) SELECT 1 UNION ALL TABLE c"),
        ["orders"]
    );
    assert_eq!(names("SELECT 1 UNION ALL TABLE orders"), ["orders"]);
}

#[test]
fn an_explicit_collation_in_the_compared_value_fixes_no_row() {
    assert_eq!(
        names("SELECT 1 FROM accounts WHERE email = $1 COLLATE \"C\""),
        ["accounts"]
    );
    assert!(names("SELECT 1 FROM accounts WHERE email = lower($1)").is_empty());
}

#[test]
fn oversized_ctes_retain_uncapped_relation_reads() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/oversized-cte.sql"
    ));
    assert_eq!(names(sql), ["orders"]);
}

#[test]
fn oversized_cte_compaction_does_not_invent_reads_from_capped_or_opaque_sources() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/oversized-cte-mixed.sql"
    ));
    assert!(names(sql).is_empty());
}

#[test]
fn oversized_cte_compaction_preserves_key_pins_without_hiding_unpinned_reads() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/oversized-cte-pins.sql"
    ));
    assert_eq!(names(sql), ["orders"]);
}

#[test]
fn jsonpath_set_returning_functions_expand_aggregate_rows() {
    assert_eq!(
        names("SELECT jsonb_path_query(jsonb_agg(to_jsonb(o)), '$[*]') FROM orders o"),
        ["orders"]
    );
    assert!(names("SELECT 1 FROM jsonb_path_query($1::jsonb, '$[*]')").is_empty());
}

#[test]
fn catalog_set_returning_names_do_not_classify_application_functions() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/set-returning-qualified.sql"));
    assert_eq!(names(sql), ["orders", "orders"]);
}

#[test]
fn snapshot_expansion_requires_a_caller_supplied_snapshot() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/snapshot-arguments.sql"
    ));
    assert_eq!(
        names(sql),
        ["accounts", "accounts", "accounts", "accounts", "accounts"]
    );
}

#[test]
fn caller_sized_set_returning_functions_require_caller_supplied_arguments() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/set-returning-arguments.sql"
    ));
    assert_eq!(names(sql), ["accounts", "accounts", "accounts", "accounts"]);
}

#[test]
fn fixed_one_row_catalog_functions_preserve_aggregate_caps() {
    assert!(names("SELECT pg_stat_get_recovery_prefetch(), count(*) FROM orders").is_empty());
}

#[test]
fn explicitly_temporary_schema_creation_also_shadows_bare_names() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-qualified.sql"
    ));
    assert_eq!(names(sql), ["accounts", "accounts", "accounts"]);
}

#[test]
fn temporary_identity_obeys_transaction_ddl_and_search_path_lifecycle() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-lifecycle.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let catalog = super::catalog();
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    assert_eq!(
        found,
        [
            ("accounts", 5),
            ("accounts", 10),
            ("accounts", 15),
            ("accounts", 24),
            ("orders", 25),
            ("accounts", 29),
            ("accounts", 34),
            ("accounts", 42),
            ("orders", 43),
            ("accounts", 48),
            ("accounts", 54),
            ("accounts", 64),
            ("accounts", 72),
            ("accounts", 79),
            ("accounts", 86),
            ("accounts", 98),
            ("accounts", 105),
            ("accounts", 108),
            ("accounts", 117),
        ]
        .map(|(name, line)| (name.to_string(), line))
    );
}

#[test]
fn temporary_relation_identity_tracks_source_statement_order() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-relations.sql"
    ));
    assert_eq!(
        names(sql),
        ["accounts", "accounts", "orders", "accounts", "accounts"]
    );
}

#[test]
fn on_commit_drop_removes_only_committed_temporary_identities() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-on-commit.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let catalog = super::catalog();
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    assert_eq!(
        found,
        [
            ("accounts", 6),
            ("accounts", 13),
            ("orders", 19),
            ("accounts", 27),
            ("accounts", 32)
        ]
        .map(|(name, line)| (name.to_string(), line))
    );
}

#[test]
fn temporary_into_set_operation_targets_shadow_later_reads() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-set-operations.sql"));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let catalog = super::catalog();
    let findings: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    assert_eq!(
        findings,
        [
            ("orders", 4),
            ("accounts", 7),
            ("orders", 8),
            ("orders", 11),
            ("orders", 14),
            ("accounts", 17)
        ]
        .map(|(table, line)| (table.to_string(), line))
    );
}

#[test]
fn temporary_schema_create_without_temp_keyword_shadows_catalog() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-schema-create.sql"));
    assert_eq!(
        names(sql),
        ["accounts", "accounts", "accounts", "accounts", "accounts"]
    );
}

#[test]
fn base_table_column_alias_lists_supply_no_catalog_key_pins() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/column-alias-list.sql"
    ));
    assert_eq!(names(sql), ["accounts", "accounts", "accounts", "accounts"]);
}

#[test]
fn table_arms_retain_possible_quoted_identifiers_and_cte_precedence() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-quoting.sql"
    ));
    assert_eq!(
        names(sql),
        [
            "\"Order Items\"",
            "\"Order Items\"",
            "accounts",
            "accounts",
            "accounts"
        ]
    );
}

#[test]
fn known_source_outputs_resolve_bare_columns_before_outer_pins() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projected-columns.sql"
    ));
    assert_eq!(
        names(sql),
        ["accounts", "accounts", "accounts", "accounts", "accounts"]
    );
}

#[test]
fn projected_composites_ordinality_and_join_aliases_preserve_ownership() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projected-output-regressions.sql"));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let catalog = super::catalog();
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    assert_eq!(
        found,
        [12, 13, 14, 15, 23, 24].map(|line| ("accounts".to_string(), line))
    );
}

#[test]
fn projected_source_aliases_set_arms_and_unknown_functions_preserve_ownership() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projected-column-variants.sql"));
    assert!(names(sql).is_empty());
}

#[test]
fn qualified_scalar_functions_and_unknown_record_layouts_preserve_ownership() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projected-column-coverage.sql"));
    assert_eq!(names(sql), ["accounts"]);
}

#[test]
fn unnest_without_a_column_alias_exposes_its_function_or_alias_name() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projected-unnest.sql"
    ));
    assert_eq!(names(sql), ["accounts", "accounts", "accounts", "accounts"]);
}
