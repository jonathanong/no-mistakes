use super::collect_ident_names;
use crate::codebase::postgres::parse_postgres_sql;
use sqlparser::ast::{SelectItem, SetExpr, Statement};

fn projection_names(sql: &str) -> Vec<String> {
    let Statement::Query(query) = parse_postgres_sql(sql)
        .unwrap()
        .pop()
        .unwrap()
    else {
        panic!("query");
    };
    let SetExpr::Select(select) = query.body.as_ref() else {
        panic!("select");
    };
    match &select.projection[0] {
        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
            collect_ident_names(expr)
        }
        other => panic!("projection {other:?}"),
    }
}

#[test]
fn collect_ident_names_walks_case_between_in_and_truth_tests() {
    let case_when = projection_names("SELECT CASE flag WHEN true THEN id ELSE note END");
    assert_eq!(case_when, ["flag", "id", "note"]);
    let searched = projection_names("SELECT CASE WHEN flag THEN id ELSE 0 END");
    assert!(searched.contains(&"flag".into()) && searched.contains(&"id".into()));
    assert_eq!(projection_names("SELECT id BETWEEN 1 AND 2"), ["id"]);
    assert_eq!(projection_names("SELECT flag IS TRUE"), ["flag"]);
    assert_eq!(projection_names("SELECT flag IS FALSE"), ["flag"]);
    assert_eq!(projection_names("SELECT id IN (note, 0)"), ["id", "note"]);
    assert_eq!(projection_names("SELECT ((id))"), ["id"]);
}

#[test]
fn collect_ident_names_walks_named_and_wildcard_function_args() {
    let named = projection_names("SELECT date_trunc('day', timestamp => created_at)");
    assert!(named.contains(&"created_at".into()), "{named:?}");
    assert!(projection_names("SELECT count(*)").is_empty());
}
