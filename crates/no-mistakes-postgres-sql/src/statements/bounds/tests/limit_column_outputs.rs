use super::super::limit_columns::{of_query, Relations};
use crate::parse_postgres_sql;
use crate::statements::limit::FixedCount;
use sqlparser::ast::Statement;
use std::collections::BTreeMap;

fn proved(sql: &str, aliases: &[&str]) -> Option<BTreeMap<String, Option<FixedCount>>> {
    let statements = parse_postgres_sql(sql).expect("query must parse");
    let Statement::Query(query) = statements.first().expect("one query") else {
        panic!("query must be a select");
    };
    let aliases = aliases
        .iter()
        .map(|name| (*name).to_string())
        .collect::<Vec<_>>();
    of_query(query, None, &Relations::default(), &aliases)
}

fn non_null(map: &BTreeMap<String, Option<FixedCount>>, name: &str) -> bool {
    matches!(map.get(name), Some(Some(FixedCount::NonNull)))
}

fn dropped(map: &BTreeMap<String, Option<FixedCount>>, name: &str) -> bool {
    matches!(map.get(name), Some(None))
}

#[test]
fn issue_3462_grouping_sets_drop_grouped_column_proofs() {
    let plain = proved("SELECT v.n FROM (VALUES (1)) v(n)", &[]).expect("plain values");
    assert!(non_null(&plain, "n"));

    for sql in [
        "SELECT v.n, 1 AS kept FROM (VALUES (1)) v(n) GROUP BY ROLLUP (v.n)",
        "SELECT v.n, 1 AS kept FROM (VALUES (1)) v(n) GROUP BY CUBE (v.n)",
        "SELECT v.n, 1 AS kept FROM (VALUES (1)) v(n) GROUP BY GROUPING SETS ((v.n), ())",
        "SELECT v.n, 1 AS kept FROM (VALUES (1)) v(n) GROUP BY v.n GROUPING SETS ((v.n), ())",
    ] {
        let outputs = proved(sql, &[]).unwrap_or_else(|| panic!("named outputs for {sql}"));
        assert!(dropped(&outputs, "n"), "{sql}");
        assert!(non_null(&outputs, "kept"), "{sql}");
    }
}

#[test]
fn issue_3462_set_operation_names_come_from_the_left_arm() {
    let named = proved("SELECT 1 AS n UNION ALL SELECT 2 AS other", &[]).expect("left name");
    assert_eq!(
        named.keys().cloned().collect::<Vec<_>>(),
        vec!["n".to_string()]
    );
    assert!(non_null(&named, "n"));

    let fallback = proved("SELECT 1 UNION ALL SELECT 2 AS other", &[]).expect("right fallback");
    assert_eq!(
        fallback.keys().cloned().collect::<Vec<_>>(),
        vec!["other".to_string()]
    );
    assert!(non_null(&fallback, "other"));

    assert!(proved("SELECT 1 AS n UNION ALL SELECT 2 AS other, 3 AS extra", &[]).is_none());
}

#[test]
fn issue_3462_prefix_alias_lists_rename_the_leading_columns() {
    let prefixed = proved("SELECT 1 AS n, 2 AS spare", &["cap"]).expect("prefix alias");
    assert_eq!(
        prefixed.keys().cloned().collect::<Vec<_>>(),
        vec!["cap".to_string(), "spare".to_string()]
    );
    assert!(non_null(&prefixed, "cap"));
    assert!(non_null(&prefixed, "spare"));

    let values = proved("VALUES (1, 2)", &["cap"]).expect("named prefix of unnamed values");
    assert_eq!(values.len(), 1);
    assert!(non_null(&values, "cap"));
    assert!(proved("VALUES (1, 2)", &[]).is_none());
    assert!(proved("SELECT 1 AS n, 2 AS spare", &["cap", "extra", "too_many"]).is_none());
    assert!(proved("SELECT 1 AS cap, 2 AS spare", &["spare"]).is_none());
}
