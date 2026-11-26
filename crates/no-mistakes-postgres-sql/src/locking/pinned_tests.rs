use super::{extract_locking_select_metadata, LockingSelectMetadata};
use std::collections::BTreeMap;

fn only(sql: &str) -> LockingSelectMetadata {
    let mut locks = extract_locking_select_metadata(sql).unwrap();
    assert_eq!(locks.len(), 1, "{sql}");
    locks.remove(0)
}

fn pinned(sql: &str, table: &str) -> Vec<String> {
    only(sql)
        .pinned_columns
        .expect("resolved tables")
        .remove(table)
        .unwrap_or_else(|| panic!("no pins for {table}"))
}

#[test]
fn top_level_equalities_to_bound_values_are_pinned() {
    let sql = "SELECT 1 FROM orders WHERE id = $1 AND (tenant = 5) AND $2::text = region \
               AND status IN ('a', 'b') FOR UPDATE";
    assert_eq!(pinned(sql, "orders"), ["id", "region", "tenant"]);
}

#[test]
fn equalities_that_do_not_bound_the_result_are_not_pinned() {
    for predicate in [
        "id = $1 OR status = 'a'",
        "NOT (id = $1)",
        "id <> $1",
        "id = other_id",
        "id = ANY($1)",
        "id IN ($1)",
        "id = (SELECT 1)",
        "lower(id) = $1",
    ] {
        let sql = format!("SELECT 1 FROM orders WHERE {predicate} FOR UPDATE");
        assert!(pinned(&sql, "orders").is_empty(), "{predicate}");
    }
    assert!(pinned("SELECT 1 FROM orders FOR UPDATE", "orders").is_empty());
}

#[test]
fn qualified_pins_must_name_the_locked_relation() {
    let join = "SELECT 1 FROM orders o JOIN accounts a ON a.id = o.account_id";
    let sql = format!("{join} WHERE o.id = $1 AND a.id = $2 AND orders.x = $3 FOR UPDATE OF o");
    assert_eq!(pinned(&sql, "orders"), ["id", "x"]);
    // An unqualified column is ambiguous once a second relation is joined.
    let sql = format!("{join} WHERE id = $1 AND o.status = ANY($2) FOR UPDATE OF o");
    assert!(pinned(&sql, "orders").is_empty());
    let sql = "SELECT 1 FROM shop.orders WHERE shop.orders.id = $1 AND \"Name\" = $2 FOR UPDATE";
    assert_eq!(pinned(sql, "shop.orders"), ["Name", "id"]);
}

#[test]
fn a_self_join_proves_nothing() {
    let sql = "SELECT 1 FROM orders a JOIN orders b ON true WHERE a.id = $1 FOR UPDATE";
    assert!(pinned(sql, "orders").is_empty());
}

#[test]
fn every_locked_table_gets_its_own_pins() {
    let sql = "SELECT 1 FROM orders o JOIN accounts a ON true WHERE o.id = $1 FOR UPDATE";
    let meta = only(sql);
    assert_eq!(
        meta.pinned_columns,
        Some(BTreeMap::from([
            ("accounts".to_string(), Vec::new()),
            ("orders".to_string(), vec!["id".to_string()]),
        ]))
    );
}

#[test]
fn lateral_and_derived_relations_resolve_only_when_not_locked() {
    let lateral = "SELECT 1 FROM order_lines line LEFT JOIN LATERAL (SELECT 1 FROM products p \
                   WHERE p.sku = line.sku LIMIT 1) m ON TRUE WHERE line.order_id = ANY($1)";
    assert_eq!(
        only(&format!("{lateral} FOR UPDATE OF line")).tables,
        Some(vec!["order_lines".to_string()])
    );
    // No OF clause locks every FROM item, including the lateral subquery.
    assert_eq!(only(&format!("{lateral} FOR UPDATE")).tables, None);
    // OF naming the derived relation, or an unknown name, is not a base table.
    assert_eq!(only(&format!("{lateral} FOR UPDATE OF m")).tables, None);
    assert_eq!(only(&format!("{lateral} FOR UPDATE OF nope")).tables, None);
    // A table function has no usable alias to lock, and joins like any derived relation.
    let function = "SELECT 1 FROM orders o JOIN unnest($1::int[]) AS u(id) ON u.id = o.id \
                    WHERE o.id = $2 FOR UPDATE OF o";
    assert_eq!(only(function).tables, Some(vec!["orders".to_string()]));
    let anonymous = "SELECT 1 FROM orders o, generate_series(1, 3) FOR UPDATE OF o";
    assert_eq!(only(anonymous).tables, Some(vec!["orders".to_string()]));
}

#[test]
fn a_parenthesized_column_is_still_pinned() {
    let sql = "SELECT 1 FROM orders o WHERE (o.id) = $1 AND (region) = 'eu' FOR UPDATE";
    assert_eq!(pinned(sql, "orders"), ["id", "region"]);
}

#[test]
fn a_lateral_function_joins_like_any_derived_relation() {
    let sql = "SELECT 1 FROM orders o, LATERAL generate_series(1, o.qty) AS g(n) \
               WHERE o.id = $1 FOR UPDATE OF o";
    assert_eq!(only(sql).tables, Some(vec!["orders".to_string()]));
    assert_eq!(pinned(sql, "orders"), ["id"]);
}
