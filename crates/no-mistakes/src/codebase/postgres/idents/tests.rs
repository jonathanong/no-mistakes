use super::{collect_ident_names, ident_key, insert_ident, object_name_ident};
use crate::codebase::postgres::parse_postgres_sql;
use sqlparser::ast::{Ident, ObjectName, SelectItem, SetExpr, Statement};
use std::collections::HashSet;

fn projection_names(sql: &str) -> Vec<String> {
    let Statement::Query(query) = parse_postgres_sql(sql).unwrap().pop().unwrap() else {
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
    assert_eq!(
        projection_names("SELECT name LIKE pattern"),
        ["name", "pattern"]
    );
    assert_eq!(
        projection_names("SELECT name ILIKE pattern"),
        ["name", "pattern"]
    );
}

#[test]
fn collect_ident_names_walks_named_and_wildcard_function_args() {
    let named = projection_names("SELECT date_trunc('day', timestamp => created_at)");
    assert!(named.contains(&"created_at".into()), "{named:?}");
    assert!(projection_names("SELECT count(*)").is_empty());
}

#[test]
fn ident_key_insert_and_object_name_helpers() {
    assert_eq!(ident_key(&Ident::new("Posts")), "posts");
    assert_eq!(ident_key(&Ident::with_quote('"', "Posts")), "Posts");
    let mut local = HashSet::new();
    insert_ident(&mut local, &Ident::new(""));
    assert!(local.is_empty());
    insert_ident(&mut local, &Ident::new("topics"));
    assert!(local.contains("topics"));
    assert!(object_name_ident(&ObjectName(Vec::new())).is_none());
}
