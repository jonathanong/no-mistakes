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
