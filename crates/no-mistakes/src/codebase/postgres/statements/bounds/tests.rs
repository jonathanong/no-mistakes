mod nonrecursive_ctes;
mod stored_array;
mod table_only;
use crate::codebase::postgres::statements::{
    extract_sql_statement_facts, SqlBoundFact, SqlBoundItem, SqlBoundItemKind, SqlBoundKind,
    SqlBoundQuery, SqlPinSource,
};

mod aliases;
mod array_review;
mod compact_aliases;
mod lateral_review;
mod materialized_drop;
mod physical_cascade;
mod recursive_forward;
mod repeated_begin;
mod review;
mod review_table;
mod table_review;
mod temporary_view;

fn query(bound: &SqlBoundQuery) -> String {
    let items: Vec<String> = bound.items.iter().map(item).collect();
    let cap = if bound.capped { "capped " } else { "" };
    format!("{cap}{}", items.join(" "))
}

fn item(item: &SqlBoundItem) -> String {
    let pins: Vec<String> = item
        .pins
        .iter()
        .map(|pin| {
            // `~=` is the null-safe `IS NOT DISTINCT FROM`.
            let operator = if pin.null_safe { "~=" } else { "=" };
            match &pin.source {
                SqlPinSource::Value => format!("{}{operator}value", pin.column),
                SqlPinSource::StoredArray(items) => {
                    format!("{}{operator}stored-array#{items:?}", pin.column)
                }
                SqlPinSource::Items(items) => {
                    let items: Vec<String> = items.iter().map(usize::to_string).collect();
                    format!("{}{operator}#{}", pin.column, items.join(","))
                }
                SqlPinSource::Array {
                    items,
                    scalar_columns,
                    indexed_columns: _,
                    cast_types: _,
                } => {
                    format!("{}{operator}array#{items:?}:{scalar_columns:?}", pin.column)
                }
                SqlPinSource::Query(bound) => format!("{}{operator}({})", pin.column, query(bound)),
            }
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
        SqlBoundItemKind::Opaque => format!("opaque{pins}"),
    }
}

pub(super) fn facts(sql: &str) -> Vec<SqlBoundFact> {
    extract_sql_statement_facts(sql).bounds
}

pub(super) fn shape(sql: &str) -> Vec<String> {
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
        ["select: t[k~=value]"]
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
        ["select: (() ((opaque)))"]
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
        ["delete: t[id=value]", "select: (opaque)"]
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
    found[0].map_lines(&|line, _| line + 10);
    assert_eq!(lines(&found[0]), [12, 13, 14]);
}

#[test]
fn expressions_over_other_items_and_scalar_subqueries_are_values() {
    assert_eq!(
        shape("SELECT 1 FROM a, b WHERE a.id IN (b.x, 5) AND b.id = $1"),
        ["select: a[id=#1] b[id=value]"]
    );
    assert_eq!(
        shape("SELECT 1 FROM a WHERE a.f = EXISTS (SELECT 1) AND a.g = (SELECT 2)"),
        ["select: a[f=value g=value]"]
    );
    assert_eq!(shape("(SELECT id FROM t LIMIT 5)"), ["select: capped t"]);
    assert_eq!(shape("SELECT id FROM t LIMIT (NULL)"), ["select: t"]);
    assert_eq!(shape("SELECT id FROM t LIMIT (4)"), ["select: capped t"]);
}

#[test]
fn derived_table_lines_map_too() {
    let mut found = facts("SELECT 1\nFROM (SELECT 1\n  FROM a) d");
    let lines = |fact: &SqlBoundFact| match &fact.query.items[0].kind {
        SqlBoundItemKind::Query(inner) => {
            vec![fact.line, fact.query.items[0].line, inner.items[0].line]
        }
        _ => Vec::new(),
    };
    assert_eq!(lines(&found[0]), [1, 2, 3]);
    found[0].map_lines(&|line, _| line * 10);
    assert_eq!(lines(&found[0]), [10, 20, 30]);
}

#[test]
fn every_item_records_where_it_starts() {
    let found = facts("SELECT *\nFROM accounts a, orders");
    let fact = &found[0];
    assert_eq!((fact.line, fact.column), (1, 1));
    let at: Vec<(usize, usize)> = fact
        .query
        .items
        .iter()
        .map(|item| (item.line, item.column))
        .collect();
    assert_eq!(at, [(2, 6), (2, 18)]);
    // The mapping sees the column too, which decides the owning operand of recovered SQL.
    let seen = std::cell::RefCell::new(Vec::new());
    let mut mapped = found.clone();
    mapped[0].map_lines(&|line, column| {
        seen.borrow_mut().push((line, column));
        line
    });
    assert_eq!(seen.into_inner(), [(1, 1), (2, 6), (2, 18)]);
}

#[test]
fn an_interpolation_recovered_from_a_template_is_a_bind() {
    // `${id}` reaches the facts as `sql_placeholder_N`: a value, never a column.
    assert_eq!(
        shape("SELECT id FROM images WHERE id = sql_placeholder_1"),
        ["select: images[id=value]"]
    );
    assert_eq!(
        shape("SELECT id FROM images WHERE sql_placeholder_1::uuid = id"),
        ["select: images[id=value]"]
    );
    assert_eq!(
        shape("SELECT id FROM images WHERE id = ANY(sql_placeholder_1)"),
        ["select: images[id=value]"]
    );
    assert_eq!(
        shape("SELECT 1 FROM a JOIN b ON b.a_id = a.id WHERE a.id = sql_placeholder_1 AND b.k IN (sql_placeholder_2, 3)"),
        ["select: a[id=#1 id=value] b[a_id=#0 k=value]"]
    );
    // A bind never names a column of the item it is compared with.
    assert_eq!(
        shape(
            "SELECT 1 FROM a, b WHERE a.id = sql_placeholder_1 AND b.id = sql_placeholder_1 + a.n"
        ),
        ["select: a[id=value] b[id=#0]"]
    );
}

#[test]
fn a_null_safe_comparison_is_recorded_as_such() {
    assert_eq!(
        shape("SELECT 1 FROM t WHERE t.k IS NOT DISTINCT FROM sql_placeholder_1 AND t.j = $1"),
        ["select: t[k~=value j=value]"]
    );
}

#[test]
fn only_a_built_in_aggregate_caps_a_query() {
    assert_eq!(shape("SELECT count(*) FROM t"), ["select: capped t"]);
    assert_eq!(
        shape("SELECT Pg_Catalog.Count(*) FROM t"),
        ["select: capped t"]
    );
    // A function of another schema is an ordinary function, called once per row.
    assert_eq!(shape("SELECT app.count(id) FROM t"), ["select: t"]);
    assert_eq!(shape("SELECT db.app.count(id) FROM t"), ["select: t"]);
    assert_eq!(shape("SELECT \"count\"(id) FROM t"), ["select: capped t"]);
    assert_eq!(shape("SELECT \"Count\"(id) FROM t"), ["select: t"]);
}

#[test]
fn fetch_caps_unless_it_returns_ties_or_a_share() {
    assert_eq!(
        shape("SELECT 1 FROM t ORDER BY k FETCH FIRST 5 ROWS ONLY"),
        ["select: capped t"]
    );
    assert_eq!(
        shape("SELECT 1 FROM t ORDER BY k FETCH FIRST ROW ONLY"),
        ["select: capped t"]
    );
    assert_eq!(
        shape("SELECT 1 FROM t ORDER BY k FETCH FIRST 5 ROWS WITH TIES"),
        ["select: t"]
    );
}

#[test]
fn table_is_a_select_of_its_relation() {
    // The parser accepts TABLE only as an arm of a set operation.
    assert_eq!(
        shape("SELECT 1 FROM a UNION TABLE orders"),
        ["select: (a) (orders)"]
    );
    assert_eq!(
        shape("SELECT 1 FROM a UNION ALL TABLE public.orders LIMIT 5"),
        ["select: capped (a) (public.orders)"]
    );
}

#[test]
fn join_using_pins_the_single_item_on_each_side() {
    assert_eq!(
        shape("SELECT 1 FROM a JOIN b USING (id) WHERE a.id = $1"),
        ["select: a[id=#1 id=value] b[id=#0]"]
    );
    // An outer join pins only the side it can null-extend.
    assert_eq!(
        shape("SELECT 1 FROM a LEFT JOIN b USING (id)"),
        ["select: a b[id=#0]"]
    );
    assert_eq!(
        shape("SELECT 1 FROM a RIGHT JOIN b USING (id)"),
        ["select: a[id=#1] b"]
    );
    assert_eq!(
        shape("SELECT 1 FROM a FULL JOIN b USING (id)"),
        ["select: a b"]
    );
    // Which item of a longer left side owns the column cannot be told.
    assert_eq!(
        shape("SELECT 1 FROM a JOIN b ON a.x = b.x JOIN c USING (id)"),
        ["select: a[x=#1] b[x=#0] c"]
    );
    // A comma list is not a chain: the join binds b and c only.
    assert_eq!(
        shape("SELECT 1 FROM a, b JOIN c USING (id)"),
        ["select: a b[id=#2] c[id=#1]"]
    );
}

#[test]
fn only_a_subquery_independent_of_the_row_is_a_pin() {
    assert_eq!(
        shape("DELETE FROM t WHERE t.id IN (SELECT t.id)"),
        ["delete: t"]
    );
    assert_eq!(
        shape("DELETE FROM t WHERE id IN (SELECT id)"),
        ["delete: t"]
    );
    assert_eq!(
        shape("DELETE FROM t WHERE t.id = (SELECT t.id)"),
        ["delete: t"]
    );
    assert_eq!(
        shape("DELETE FROM t WHERE t.id = ANY (SELECT u.t_id FROM u WHERE u.t_id = t.id)"),
        ["delete: t"]
    );
    // An inner relation shadows an outer name, and a bare column has the inner relation.
    assert_eq!(
        shape("DELETE FROM t WHERE id IN (SELECT id FROM t LIMIT 5)"),
        ["delete: t[id=(capped t)]"]
    );
    assert_eq!(
        shape("DELETE FROM t s WHERE s.id IN (SELECT s.id FROM t s LIMIT 5)"),
        ["delete: t[id=(capped t)]"]
    );
    assert_eq!(
        shape("DELETE FROM t WHERE id = (SELECT max(id) FROM t)"),
        ["delete: t[id=value]"]
    );
}

#[test]
fn a_function_that_differs_per_row_is_not_a_value() {
    assert_eq!(
        shape("DELETE FROM t WHERE id = nextval('s')"),
        ["delete: t"]
    );
    assert_eq!(
        shape("DELETE FROM t WHERE id = pg_catalog.random()"),
        ["delete: t"]
    );
    assert_eq!(
        shape("DELETE FROM t WHERE id = ANY(ARRAY[gen_random_uuid()])"),
        ["delete: t"]
    );
    assert_eq!(
        shape("DELETE FROM t WHERE id = lower($1)"),
        ["delete: t[id=value]"]
    );
    assert_eq!(
        shape("DELETE FROM t WHERE id = now()"),
        ["delete: t[id=value]"]
    );
}

#[test]
fn a_dml_target_is_never_a_cte() {
    assert_eq!(
        shape("WITH t AS (SELECT 1 AS id) DELETE FROM t"),
        ["delete: t"]
    );
    assert_eq!(
        shape("WITH c AS (SELECT 1) UPDATE t SET x = 1 FROM c"),
        ["update: t ()"]
    );
}

mod order_by_kind;
