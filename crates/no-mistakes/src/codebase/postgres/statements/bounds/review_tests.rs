use super::tests::shape;

#[test]
fn only_a_set_returning_built_in_over_given_arguments_is_sized_by_its_caller() {
    assert_eq!(
        shape("SELECT 1 FROM unnest($1::int[]) AS x"),
        ["select: other"]
    );
    assert_eq!(
        shape("SELECT 1 FROM generate_series(1, 10) g"),
        ["select: other"]
    );
    assert_eq!(
        shape("SELECT 1 FROM jsonb_array_elements($1::jsonb) AS e"),
        ["select: other"]
    );
    // Any other function can return rows from anywhere, and so can an array taken from a query.
    assert_eq!(
        shape("SELECT 1 FROM get_all_accounts() f"),
        ["select: opaque"]
    );
    assert_eq!(
        shape("SELECT 1 FROM unnest(ARRAY(SELECT id FROM t)) u"),
        ["select: opaque"]
    );
    // A VALUES list is its own text.
    assert_eq!(
        shape("SELECT 1 FROM (VALUES (1), (2)) v"),
        ["select: (other)"]
    );
}

#[test]
fn a_recursive_reference_bounds_nothing() {
    assert_eq!(
        shape(
            "WITH RECURSIVE r AS (SELECT id FROM n WHERE id = $1 \
             UNION ALL SELECT n.id FROM n JOIN r ON n.parent = r.id) SELECT 1 FROM r"
        ),
        ["select: ((n[id=value]) (n[parent=#1] (opaque)))"]
    );
}

#[test]
fn an_aggregate_in_having_or_among_the_rare_built_ins_caps_a_query() {
    assert_eq!(
        shape("SELECT 1 FROM t HAVING count(*) > 0"),
        ["select: capped t"]
    );
    for call in [
        "corr(a, b)",
        "covar_pop(a, b)",
        "regr_slope(a, b)",
        "json_arrayagg(a)",
    ] {
        assert_eq!(
            shape(&format!("SELECT {call} FROM t")),
            ["select: capped t"],
            "{call}"
        );
    }
    // A set-returning function in the select list turns one aggregate row into many.
    assert_eq!(
        shape("SELECT generate_series(1, count(*)) FROM t"),
        ["select: t"]
    );
    assert_eq!(shape("SELECT unnest(array_agg(id)) FROM t"), ["select: t"]);
    // A window function is not an aggregate, whatever it is called.
    assert_eq!(shape("SELECT count(*) OVER () FROM t"), ["select: t"]);
}

#[test]
fn only_a_fixed_count_caps_a_query() {
    for limit in [
        "LIMIT 5",
        "LIMIT $1",
        "LIMIT $1::int",
        "LIMIT (5)",
        "LIMIT LEAST($1, 100)",
        "LIMIT COALESCE($1, 100)",
        "LIMIT 2 * $1",
        "LIMIT sql_placeholder_1",
        "FETCH FIRST ROW ONLY",
        "FETCH FIRST $1 ROWS ONLY",
    ] {
        assert_eq!(
            shape(&format!("SELECT 1 FROM t {limit}")),
            ["select: capped t"],
            "{limit}"
        );
    }
    for limit in [
        "LIMIT NULL",
        "LIMIT NULL::bigint",
        "LIMIT ALL",
        "LIMIT (SELECT count(*) FROM t)",
        "LIMIT (SELECT NULL::bigint)",
        "LIMIT COALESCE($1, NULL)",
        "LIMIT n",
    ] {
        assert_eq!(
            shape(&format!("SELECT 1 FROM t {limit}")),
            ["select: t"],
            "{limit}"
        );
    }
}

#[test]
fn a_query_inside_copy_is_judged_like_a_select() {
    assert_eq!(shape("COPY (SELECT 1 FROM t) TO STDOUT"), ["select: t"]);
    assert_eq!(
        shape("COPY (SELECT 1 FROM t LIMIT 5) TO STDOUT"),
        ["select: capped t"]
    );
    assert!(shape("COPY t TO STDOUT").is_empty());
}

#[test]
fn array_from_a_query_is_not_a_value() {
    assert_eq!(
        shape("DELETE FROM t WHERE id = ANY(ARRAY(SELECT id FROM t))"),
        ["delete: t"]
    );
    assert_eq!(
        shape("DELETE FROM t WHERE id = ANY(ARRAY[$1, $2])"),
        ["delete: t[id=value]"]
    );
}
