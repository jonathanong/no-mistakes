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
    let pinned = |select: &str| source(&insert(select)).pinned_relation;
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
fn pins_require_one_plain_relation_and_constant_values() {
    let none = |select: &str| {
        assert!(
            source(&insert(select)).pinned_relation.is_none(),
            "{select}"
        )
    };
    none("SELECT id, note FROM orders");
    none("SELECT id, note FROM orders WHERE id > $1");
    none("SELECT id, note FROM orders WHERE id = $1 OR id = $2");
    none("SELECT id, note FROM orders WHERE id = other_id");
    none("SELECT id, note FROM orders WHERE id = (SELECT 1)");
    none("SELECT o.id, o.note FROM orders o WHERE orders.id = $1");
    none("SELECT o.id, o.note FROM orders o WHERE x.o.id = $1");
    none("SELECT id, note FROM orders a, other b WHERE a.id = $1");
    none("SELECT a.id, a.note FROM orders a JOIN other b ON true WHERE a.id = $1");
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
    assert!(shadowed[0].source.pinned_relation.is_none());
    let unrelated = analyze_conflict_inserts(
        "WITH picked AS (SELECT 1 AS id) INSERT INTO items (id, note) SELECT o.id, 1 FROM orders o WHERE o.id = $1 ON CONFLICT (id) DO NOTHING",
    )
    .unwrap();
    assert!(unrelated[0].source.pinned_relation.is_some());
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
