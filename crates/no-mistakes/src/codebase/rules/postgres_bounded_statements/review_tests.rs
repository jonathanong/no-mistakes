use super::tests::names;

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
