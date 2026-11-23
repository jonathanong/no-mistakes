use super::*;

fn source(sql: &str) -> SqlInsertSourceShape {
    let mut inserts = analyze_conflict_inserts(sql).unwrap();
    assert_eq!(inserts.len(), 1, "{sql}");
    inserts.remove(0).source
}

fn insert(select: &str) -> String {
    format!("INSERT INTO items (id, note) {select} ON CONFLICT (id) DO NOTHING")
}

#[test]
fn from_less_scalar_selects_are_single_row() {
    for select in [
        "SELECT $1, $2 WHERE true",
        "SELECT $1::uuid, lower($2) WHERE EXISTS (SELECT 1 FROM orders WHERE id = $1)",
        "SELECT coalesce($1, 'x'), now() WHERE true",
        "SELECT 1 + 2, (SELECT max(id) FROM other) WHERE true",
        "SELECT -1, upper(lower($1)) WHERE true",
        "SELECT gen_random_uuid(), (nullif($1, '')) WHERE true",
        "(SELECT $1, $2 WHERE true)",
    ] {
        assert!(!source(&insert(select)).multi_row, "{select}");
    }
}

#[test]
fn from_less_selects_that_may_expand_stay_multi_row() {
    for select in [
        "SELECT unnest($1::uuid[]) AS id, $2 WHERE true",
        "SELECT $1, generate_series(1, $2) AS note WHERE true",
        "SELECT id, $1 WHERE true",
        "SELECT * WHERE true",
        "SELECT sum($1) OVER () AS id, $2 WHERE true",
        "SELECT lower(note => $1), $2 WHERE true",
        "SELECT lower((SELECT 1)), 1 IS NULL WHERE true",
        "SELECT $1 FROM unnest($2::uuid[]) AS input(id)",
        "SELECT $1 UNION ALL SELECT $2 WHERE true",
    ] {
        assert!(source(&insert(select)).multi_row, "{select}");
    }
}

#[test]
fn literal_limits_bound_the_source_to_one_row() {
    for select in [
        "SELECT id, note FROM staged LIMIT 1",
        "SELECT id, note FROM staged LIMIT 0",
        "SELECT id, note FROM staged ORDER BY id FETCH FIRST ROW ONLY",
        "SELECT id, note FROM staged FETCH FIRST 1 ROWS ONLY",
        "SELECT id, note FROM staged UNION ALL SELECT id, note FROM other LIMIT 1",
    ] {
        assert!(!source(&insert(select)).multi_row, "{select}");
    }
    for select in [
        "SELECT id, note FROM staged LIMIT 2",
        "SELECT id, note FROM staged LIMIT $1",
        "SELECT id, note FROM staged LIMIT ALL",
        "SELECT id, note FROM staged OFFSET 1",
        "SELECT id, note FROM staged FETCH FIRST 1 PERCENT ROWS ONLY",
        "SELECT id, note FROM staged FETCH FIRST 2 ROWS ONLY",
    ] {
        assert!(source(&insert(select)).multi_row, "{select}");
    }
}

#[test]
fn pinned_relations_capture_constant_equalities() {
    let pinned = |select: &str| {
        source(&insert(select))
            .pinned_source
            .map(|mut s| s.relations.remove(0))
    };
    let relation = pinned("SELECT o.id, o.total FROM orders o WHERE o.id = $1").unwrap();
    assert_eq!(relation.table, "orders");
    assert_eq!(relation.columns, ["id"]);

    // Unaliased, schema-qualified, quoted, reversed and parenthesized forms.
    assert_eq!(
        pinned("SELECT orders.id, 1 FROM public.orders WHERE (7 = orders.id)")
            .unwrap()
            .columns,
        ["id"]
    );
    assert_eq!(
        pinned("SELECT public.orders.id, 1 FROM public.orders WHERE public.orders.id = $1")
            .unwrap()
            .columns,
        ["id"]
    );
    assert_eq!(
        pinned("SELECT \"Id\", 1 FROM \"Orders\" WHERE \"Id\" = $1::uuid")
            .unwrap()
            .columns,
        ["\"Id\""]
    );
    let both = pinned(
        "SELECT l.id, lower(l.note) FROM lines AS l WHERE l.a = $1 AND (l.b = 'x' AND l.c > 1)",
    )
    .unwrap();
    assert_eq!(both.columns, ["a", "b"]);
}

#[test]
fn pinned_joins_capture_inner_and_cross_join_equalities() {
    let pinned = |select: &str| source(&insert(select)).pinned_source;
    let joined = pinned(
        "SELECT o.id, l.line_no FROM orders o JOIN order_lines l ON l.order_id = o.id WHERE o.id = $1 AND l.line_no = $2",
    )
    .unwrap();
    assert_eq!(joined.relations.len(), 2);
    assert_eq!(joined.relations[0].table, "orders");
    assert_eq!(joined.relations[0].columns, ["id"]);
    assert_eq!(joined.relations[1].table, "order_lines");
    assert_eq!(joined.relations[1].columns, ["line_no"]);
    assert_eq!(
        joined.join_equalities,
        [SqlPinnedJoinEquality {
            left_relation: 1,
            left_column: "order_id".to_string(),
            right_relation: 0,
            right_column: "id".to_string(),
        }]
    );

    let cross = pinned(
        "SELECT o.id, l.line_no FROM orders o CROSS JOIN order_lines l WHERE o.id = $1 AND l.order_id = o.id AND l.line_no = $2",
    )
    .unwrap();
    assert_eq!(cross.join_equalities.len(), 1);

    let comma = pinned(
        "SELECT o.id, l.line_no FROM orders o, order_lines l WHERE o.id = $1 AND l.order_id = o.id AND l.line_no = $2",
    )
    .unwrap();
    assert_eq!(comma.join_equalities.len(), 1);

    // Self-joins keep distinct relation indices.
    let self_join = pinned(
        "SELECT a.id, b.id FROM orders a INNER JOIN orders b ON b.id = a.parent_id WHERE a.id = $1",
    )
    .unwrap();
    assert_eq!(self_join.relations.len(), 2);
    assert_eq!(self_join.join_equalities[0].left_relation, 1);
    assert_eq!(self_join.join_equalities[0].right_relation, 0);
}

#[test]
fn pins_require_plain_inner_relations_and_constant_or_join_equalities() {
    let none = |select: &str| assert!(source(&insert(select)).pinned_source.is_none(), "{select}");
    none("SELECT id, note FROM orders");
    none("SELECT id, note FROM orders WHERE id > $1");
    none("SELECT id, note FROM orders WHERE id = $1 OR id = $2");
    none("SELECT id, note FROM orders WHERE id = other_id");
    none("SELECT id, note FROM orders WHERE id = (SELECT 1)");
    none("SELECT o.id, o.note FROM orders o WHERE orders.id = $1");
    none("SELECT o.id, o.note FROM orders o WHERE x.o.id = $1");
    none("SELECT id, note FROM orders a, other b WHERE a.id = $1");
    none("SELECT a.id, a.note FROM orders a JOIN other b ON true WHERE a.id = $1");
    none("SELECT o.id, l.line_no FROM orders o LEFT JOIN order_lines l ON l.order_id = o.id WHERE o.id = $1 AND l.line_no = $2");
    none("SELECT o.id, l.line_no FROM orders o RIGHT JOIN order_lines l ON l.order_id = o.id WHERE o.id = $1 AND l.line_no = $2");
    none("SELECT o.id, l.line_no FROM orders o JOIN order_lines l USING (id) WHERE o.id = $1");
    none("SELECT o.id, l.line_no FROM orders o JOIN order_lines l ON l.order_id = o.id OR l.line_no = $2 WHERE o.id = $1");
    none("SELECT o.id, l.line_no FROM orders o JOIN order_lines l ON l.order_id = o.id WHERE id = $1 AND line_no = $2");
    none("SELECT o.id, o.note FROM orders AS o(id, note) WHERE o.id = $1");
    none("SELECT o.id, o.note FROM orders o WHERE o.id = $1 GROUP BY GROUPING SETS ((o.id, o.note), ())");
    none("SELECT o.id, o.note FROM orders o JOIN orders o ON o.id = o.id WHERE o.id = $1");
    none("SELECT id, note FROM unnest($1::uuid[]) AS u(id, note) WHERE id = $1");
    none("SELECT id, note FROM (SELECT 1) AS d WHERE id = $1");
    none("SELECT unnest(tags), note FROM orders WHERE id = $1");
    none("SELECT * FROM orders WHERE id = $1");
    none("WITH x AS (SELECT 1) SELECT id, note FROM orders WHERE id = $1");
    none("SELECT id, note FROM orders UNION ALL SELECT id, note FROM other");
    none("SELECT $1, $2 WHERE true");
}

#[test]
fn ctes_shadow_pinned_catalog_tables() {
    let shadowed = analyze_conflict_inserts(
        "WITH orders AS (SELECT 1 AS id) INSERT INTO items (id, note) SELECT o.id, 1 FROM orders o WHERE o.id = $1 ON CONFLICT (id) DO NOTHING",
    )
    .unwrap();
    assert!(shadowed[0].source.pinned_source.is_none());
    let shadowed_join = analyze_conflict_inserts(
        "WITH order_lines AS (SELECT 1 AS order_id, 1 AS line_no) INSERT INTO items (id, note) SELECT o.id, l.line_no FROM orders o JOIN order_lines l ON l.order_id = o.id WHERE o.id = $1 AND l.line_no = $2 ON CONFLICT (id) DO NOTHING",
    )
    .unwrap();
    assert!(shadowed_join[0].source.pinned_source.is_none());
    let unrelated = analyze_conflict_inserts(
        "WITH picked AS (SELECT 1 AS id) INSERT INTO items (id, note) SELECT o.id, 1 FROM orders o WHERE o.id = $1 ON CONFLICT (id) DO NOTHING",
    )
    .unwrap();
    assert!(unrelated[0].source.pinned_source.is_some());
}

#[test]
fn select_lists_support_positional_order_references() {
    let shape = source(&insert("SELECT i.id, lower(i.note) AS n FROM input i"));
    assert_eq!(
        shape.select_list,
        Some(vec!["i.id".to_string(), "lower(i.note)".to_string()])
    );
    assert_eq!(source(&insert("SELECT * FROM input")).select_list, None);
    assert_eq!(
        source(&insert(
            "SELECT id, note FROM a UNION ALL SELECT id, note FROM b"
        ))
        .select_list,
        None
    );
}

#[test]
fn constant_expressions_are_literals_and_parameters_only() {
    for constant in ["$1", "$1::uuid", "'x'", "(1)", "-1", "NULL", "($1)::text"] {
        assert!(expression_is_constant(constant), "{constant}");
    }
    for varying in [
        "id",
        "i.id",
        "lower($1)",
        "$1 + 1",
        "random()",
        "(SELECT 1)",
        "lower(",
    ] {
        assert!(!expression_is_constant(varying), "{varying}");
    }
}

#[test]
fn recovered_interpolations_are_bound_values_only_at_recovered_positions() {
    // Line 1 holds the INSERT; the select list `sql_placeholder_1, sql_placeholder_2` starts at
    // column 40. Only identifiers at a recovered position are binds.
    let sql = "INSERT INTO items (id, note) SELECT sql_placeholder_1, sql_placeholder_2 WHERE true ON CONFLICT (id) DO NOTHING";
    let start = sql.find("sql_placeholder_1").unwrap() as u32 + 1;
    let second = sql.find("sql_placeholder_2").unwrap() as u32 + 1;
    let shape = |binds: &[(u32, u32)]| {
        analyze_conflict_inserts_with_binds(sql, binds)
            .unwrap()
            .remove(0)
            .source
    };
    assert!(!shape(&[(1, start), (1, second)]).multi_row);
    assert!(
        shape(&[(1, start)]).multi_row,
        "an unrecovered identifier is a column"
    );
    assert!(shape(&[]).multi_row);
    assert!(analyze_conflict_inserts(sql).unwrap()[0].source.multi_row);

    // CURRENT_TIMESTAMP and friends are scalars.
    let sql = "INSERT INTO items (id, note) SELECT 1, CURRENT_TIMESTAMP WHERE true ON CONFLICT (id) DO NOTHING";
    assert!(!analyze_conflict_inserts(sql).unwrap()[0].source.multi_row);

    // A recovered interpolation pins a unique key on either side of the equality, but is never
    // the column side.
    let sql = "INSERT INTO items (id, note) SELECT o.id, 1 FROM orders o WHERE sql_placeholder_1 = o.id ON CONFLICT (id) DO NOTHING";
    let at = sql.find("sql_placeholder_1").unwrap() as u32 + 1;
    let pinned = analyze_conflict_inserts_with_binds(sql, &[(1, at)])
        .unwrap()
        .remove(0);
    assert_eq!(
        pinned.source.pinned_source.unwrap().relations[0].columns,
        ["id"]
    );
    let pinned = analyze_conflict_inserts(sql).unwrap().remove(0);
    assert!(pinned.source.pinned_source.is_none());
}
