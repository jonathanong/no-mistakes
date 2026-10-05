use super::*;
use crate::codebase::postgres::SqlInsertSourceShape;

fn catalog(scenario: &str) -> SchemaCatalog {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/conflict-ordering/cases")
        .join(scenario)
        .join("schema.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn targets(scenario: &str, sql: &str) -> Vec<String> {
    findings_for_sql("query.ts", 1, sql, &catalog(scenario), true)
        .into_iter()
        .filter_map(|finding| finding.target)
        .collect()
}

fn key(expression: &str) -> CanonicalOrderKey {
    CanonicalOrderKey {
        expression: expression.to_string(),
        ascending: true,
        nulls_first: false,
    }
}

#[test]
fn catalog_unique_keys_decide_whether_equalities_pin_one_row() {
    let catalog = catalog("fail-single-row-lookalikes");
    let columns = |names: &[&str]| names.iter().map(|n| n.to_string()).collect::<Vec<_>>();
    assert!(catalog.columns_pin_one_row("orders", &columns(&["id"])));
    assert!(catalog.columns_pin_one_row("orders", &columns(&["ID", "total"])));
    assert!(catalog.columns_pin_one_row("order_lines", &columns(&["line_no", "order_id"])));
    assert!(!catalog.columns_pin_one_row("order_lines", &columns(&["order_id"])));
    assert!(!catalog.columns_pin_one_row("orders", &columns(&["total"])));
    // A partial unique index only covers some rows; an unknown table proves nothing.
    assert!(!catalog.columns_pin_one_row("drafts", &columns(&["id"])));
    assert!(!catalog.columns_pin_one_row("missing", &columns(&["id"])));
    assert!(!catalog.columns_pin_one_row("orders", &[]));
}

#[test]
fn single_row_sources_need_no_canonical_order() {
    let scenario = "fail-single-row-lookalikes";
    let pinned = "INSERT INTO order_archive (order_id, total) SELECT o.id, o.total FROM orders o WHERE o.id = $1 ON CONFLICT (order_id) DO NOTHING";
    assert!(targets(scenario, pinned).is_empty());
    let lookalike = pinned.replace("o.id = $1", "o.total = $1");
    assert_eq!(targets(scenario, &lookalike), ["missing-canonical-order"]);
}

#[test]
fn constant_keys_neither_require_nor_break_the_order() {
    let scenario = "pass-order-by-forms";
    let insert = |select: &str| {
        format!("INSERT INTO members (account_id, user_id) {select} ON CONFLICT (account_id, user_id) DO NOTHING")
    };
    let source = "FROM unnest($2::uuid[]) AS input(user_id)";
    for order in [
        "ORDER BY input.user_id",
        "ORDER BY $1::uuid, input.user_id",
        "ORDER BY 2",
    ] {
        let sql = insert(&format!(
            "SELECT $1::uuid AS account_id, input.user_id {source} {order}"
        ));
        assert!(targets(scenario, &sql).is_empty(), "{order}");
    }
    // A parameter without a cast is still constant, whichever way ORDER BY spells it.
    let sql = insert(&format!(
        "SELECT $1, input.user_id {source} ORDER BY $1::uuid, 2"
    ));
    assert!(targets(scenario, &sql).is_empty());
    // Only the varying key is required, and it is reported without the constant.
    let sql = insert(&format!("SELECT $1::uuid, input.user_id {source}"));
    let findings = findings_for_sql("query.ts", 1, &sql, &catalog(scenario), true);
    assert_eq!(findings.len(), 1);
    assert!(findings[0]
        .message
        .ends_with("input.user_id ASC NULLS LAST"));
    let sql = insert(&format!(
        "SELECT $1::uuid, input.user_id {source} ORDER BY input.user_id DESC"
    ));
    assert_eq!(targets(scenario, &sql), ["noncanonical-order"]);
    // Every arbiter key constant: all rows share one key, so no order exists to require.
    let sql = "INSERT INTO members (account_id, user_id) SELECT $1::uuid, $2::uuid FROM generate_series(1, 3) ON CONFLICT (account_id, user_id) DO NOTHING";
    assert!(targets(scenario, sql).is_empty());
}

#[test]
fn positional_references_resolve_through_the_select_list() {
    let source = SqlInsertSourceShape {
        select_list: Some(vec!["i.id".to_string(), "lower(i.note)".to_string()]),
        ..Default::default()
    };
    let resolved =
        order::resolve_references(&[key("2"), key("1"), key("3"), key("0"), key("x")], &source);
    let expressions: Vec<_> = resolved.iter().map(|key| key.expression.as_str()).collect();
    assert_eq!(expressions, ["lower(i.note)", "i.id", "3", "0", "x"]);
    // Without a select list (wildcards, set operations) a position stays unresolved.
    let wildcard = SqlInsertSourceShape::default();
    assert_eq!(
        order::resolve_references(&[key("1")], &wildcard)[0].expression,
        "1"
    );
    // Constants are dropped; columns, calls and unparsable text are kept.
    let kept = order::without_constants(&[
        key("$1"),
        key("'x'"),
        key("id"),
        key("now()"),
        key("lower("),
    ]);
    assert_eq!(kept.len(), 3);
}
