use crate::codebase::postgres::statements::{
    extract_sql_statement_facts, SqlBoundFact, SqlBoundItem, SqlBoundItemKind, SqlBoundKind,
    SqlBoundQuery, SqlPinSource,
};

fn query(bound: &SqlBoundQuery) -> String {
    let items: Vec<String> = bound.items.iter().map(item).collect();
    let cap = if bound.capped { "capped " } else { "" };
    format!("{cap}{}", items.join(" "))
}

fn item(item: &SqlBoundItem) -> String {
    let pins: Vec<String> = item
        .pins
        .iter()
        .map(|pin| match &pin.source {
            SqlPinSource::Value => format!("{}=value", pin.column),
            SqlPinSource::Items(items) => {
                let items: Vec<String> = items.iter().map(usize::to_string).collect();
                format!("{}=#{}", pin.column, items.join(","))
            }
            SqlPinSource::Query(bound) => format!("{}=({})", pin.column, query(bound)),
        })
        .collect();
    let pins = if pins.is_empty() {
        String::new()
    } else {
        format!("[{}]", pins.join(" "))
    };
    match &item.kind {
        SqlBoundItemKind::Table(table) => format!("{table}{pins}"),
        SqlBoundItemKind::Query(bound) => format!("({}){pins}", query(bound)),
        SqlBoundItemKind::Other => format!("other{pins}"),
    }
}

fn facts(sql: &str) -> Vec<SqlBoundFact> {
    extract_sql_statement_facts(sql).bounds
}

fn shape(sql: &str) -> Vec<String> {
    facts(sql)
        .iter()
        .map(|fact| {
            let kind = match fact.kind {
                SqlBoundKind::Select => "select",
                SqlBoundKind::Update => "update",
                SqlBoundKind::Delete => "delete",
            };
            format!("{kind}: {}", query(&fact.query))
        })
        .collect()
}

#[test]
fn equality_conjuncts_pin_columns_to_values() {
    assert_eq!(
        shape("SELECT name FROM accounts WHERE id = $1"),
        ["select: accounts[id=value]"]
    );
    assert_eq!(
        shape("SELECT 1 FROM accounts a WHERE a.id = 7 AND a.tenant = lower($2) AND a.x > 1"),
        ["select: accounts[id=value tenant=value]"]
    );
    assert_eq!(
        shape("SELECT 1 FROM t WHERE $1 IS NOT DISTINCT FROM t.k"),
        ["select: t[k=value]"]
    );
    // A caller-sized list or array pins the column too.
    assert_eq!(
        shape("SELECT id FROM orders WHERE id = ANY($1::uuid[])"),
        ["select: orders[id=value]"]
    );
    assert_eq!(
        shape("SELECT id FROM orders WHERE id IN (1, $1)"),
        ["select: orders[id=value]"]
    );
    // Alternatives, ranges and negations do not pin.
    assert_eq!(
        shape("SELECT 1 FROM t WHERE id = $1 OR id = $2"),
        ["select: t"]
    );
    assert_eq!(
        shape("SELECT 1 FROM t WHERE id > $1 AND id NOT IN (1, 2)"),
        ["select: t"]
    );
    // A column compared with its own table's column is not a pin.
    assert_eq!(shape("SELECT 1 FROM t WHERE t.a = t.b"), ["select: t"]);
}

#[test]
fn limits_and_pure_aggregates_cap_a_query() {
    assert_eq!(shape("SELECT id FROM t LIMIT $1"), ["select: capped t"]);
    assert_eq!(
        shape("SELECT id FROM t FETCH FIRST 5 ROWS ONLY"),
        ["select: capped t"]
    );
    assert_eq!(shape("SELECT id FROM t LIMIT NULL"), ["select: t"]);
    assert_eq!(shape("SELECT id FROM t LIMIT ALL"), ["select: t"]);
    assert_eq!(
        shape("SELECT COUNT(*) FROM orders WHERE account_id = $1"),
        ["select: capped orders[account_id=value]"]
    );
    assert_eq!(
        shape("SELECT coalesce(sum(total), 0) FROM orders"),
        ["select: capped orders"]
    );
    // Grouped, windowed and subquery aggregates still return many rows.
    assert_eq!(
        shape("SELECT account_id, count(*) FROM orders GROUP BY account_id"),
        ["select: orders"]
    );
    assert_eq!(
        shape("SELECT count(*) OVER () FROM orders"),
        ["select: orders"]
    );
    assert_eq!(
        shape("SELECT (SELECT count(*) FROM x) FROM orders"),
        ["select: orders"]
    );
}

#[test]
fn joins_pin_items_to_other_items_where_the_condition_restricts_them() {
    assert_eq!(
        shape("SELECT 1 FROM orders o JOIN accounts a ON a.id = o.account_id WHERE o.id = $1"),
        ["select: orders[account_id=#1 id=value] accounts[id=#0]"]
    );
    // LEFT JOIN restricts only the right side; RIGHT JOIN only the left; FULL neither.
    assert_eq!(
        shape("SELECT 1 FROM a LEFT JOIN b ON b.a_id = a.id"),
        ["select: a b[a_id=#0]"]
    );
    assert_eq!(
        shape("SELECT 1 FROM a RIGHT JOIN b ON b.a_id = a.id"),
        ["select: a[id=#1] b"]
    );
    assert_eq!(
        shape("SELECT 1 FROM a FULL JOIN b ON b.a_id = a.id"),
        ["select: a b"]
    );
    // WHERE restricts every item, even the nullable side.
    assert_eq!(
        shape("SELECT 1 FROM a LEFT JOIN b ON true WHERE b.id = $1"),
        ["select: a b[id=value]"]
    );
    // A bare column among several items has no provable owner.
    assert_eq!(shape("SELECT 1 FROM a, b WHERE id = $1"), ["select: a b"]);
    assert_eq!(
        shape("SELECT 1 FROM a, b WHERE a.id = b.id + 1 AND b.id = $1"),
        ["select: a[id=#1] b[id=value]"]
    );
    // An unknown qualifier is an outer reference, not a value.
    assert_eq!(
        shape("SELECT 1 FROM a WHERE a.id = outer_ref.id"),
        ["select: a"]
    );
}

#[test]
fn ctes_derived_tables_and_set_operations_carry_their_own_bounds() {
    assert_eq!(
        shape("WITH c AS (SELECT id FROM t ORDER BY id LIMIT $1) SELECT * FROM c"),
        ["select: (capped t)"]
    );
    assert_eq!(
        shape("SELECT * FROM (SELECT id FROM t WHERE k = $1) AS d"),
        ["select: (t[k=value])"]
    );
    assert_eq!(
        shape("SELECT id FROM a UNION ALL SELECT id FROM b LIMIT 3"),
        ["select: capped (a) (b)"]
    );
    assert_eq!(
        shape("SELECT 1 FROM unnest($1::int[]) AS x"),
        ["select: other"]
    );
    assert_eq!(
        shape("SELECT 1 FROM (VALUES (1), (2)) AS v"),
        ["select: (other)"]
    );
    // A schema-qualified name is a table even when a CTE shares the bare name.
    assert_eq!(
        shape("WITH t AS (SELECT 1) SELECT * FROM public.t, t"),
        ["select: public.t ()"]
    );
    assert_eq!(
        shape("WITH RECURSIVE r AS (SELECT 1 UNION ALL SELECT n FROM r) SELECT * FROM r"),
        ["select: (() ((other)))"]
    );
}

#[test]
fn updates_and_deletes_name_their_target_and_the_items_that_feed_it() {
    assert_eq!(
        shape("UPDATE exports SET s3_key = NULL WHERE expires_at < now()"),
        ["update: exports"]
    );
    assert_eq!(
        shape("DELETE FROM sessions WHERE id = $1"),
        ["delete: sessions[id=value]"]
    );
    let found = facts(
        "WITH c AS (SELECT id FROM exports WHERE expires_at < now() ORDER BY id LIMIT $1 \
         FOR UPDATE SKIP LOCKED) UPDATE exports SET s3_key = NULL FROM c WHERE exports.id = c.id",
    );
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].target, Some(0));
    assert_eq!(query(&found[0].query), "exports[id=#1] (capped exports)");
    assert_eq!(
        shape("DELETE FROM t USING u WHERE t.id = u.t_id AND u.id = $1"),
        ["delete: t[id=#1] u[t_id=#0 id=value]"]
    );
    assert_eq!(
        shape("DELETE FROM t WHERE id IN (SELECT id FROM t ORDER BY id LIMIT 10)"),
        ["delete: t[id=(capped t)]"]
    );
    assert_eq!(
        shape("UPDATE t SET a = 1 WHERE id = ANY (SELECT id FROM q WHERE k = $1)"),
        ["update: t[id=(q[k=value])]"]
    );
    // MySQL's LIMIT is a limit.
    assert!(facts("DELETE FROM t WHERE a = 1 LIMIT 5")[0].query.capped);
}

#[test]
fn data_modifying_ctes_are_statements_of_their_own() {
    assert_eq!(
        shape("WITH d AS (DELETE FROM t WHERE id = $1 RETURNING id) SELECT * FROM d"),
        ["delete: t[id=value]", "select: (other)"]
    );
    assert_eq!(
        shape("EXPLAIN ANALYZE UPDATE t SET a = 1 WHERE id = $1"),
        ["update: t[id=value]"]
    );
    assert!(facts("INSERT INTO t SELECT * FROM u").is_empty());
}

#[test]
fn lines_map_through_nested_items_and_pins() {
    let mut found =
        facts("\nSELECT *\n FROM a\n WHERE a.id IN (SELECT id FROM b)\n AND a.k = (SELECT 1) ");
    let lines = |fact: &SqlBoundFact| {
        let mut lines = vec![fact.line];
        for item in &fact.query.items {
            lines.push(item.line);
            for pin in &item.pins {
                if let SqlPinSource::Query(bound) = &pin.source {
                    lines.extend(bound.items.iter().map(|item| item.line));
                }
            }
        }
        lines
    };
    assert_eq!(lines(&found[0]), [2, 3, 4]);
    found[0].map_lines(&|line| line + 10);
    assert_eq!(lines(&found[0]), [12, 13, 14]);
}
