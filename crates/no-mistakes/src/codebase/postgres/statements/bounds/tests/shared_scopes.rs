use super::super::{query, Scope};
use crate::codebase::postgres::{parse_postgres_sql, SqlBoundItemKind, SqlBoundQuery};
use sqlparser::ast::Statement;
use std::{borrow::Cow, collections::BTreeSet, rc::Rc};

fn physical_tables(query: &SqlBoundQuery, tables: &mut Vec<String>) {
    for item in &query.items {
        match &item.kind {
            SqlBoundItemKind::Table(name) => tables.push(name.clone()),
            SqlBoundItemKind::Query(inner) => physical_tables(inner, tables),
            _ => {}
        }
    }
}

#[test]
fn lexical_frames_share_bounds_and_preserve_shadowed_metadata() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/lexical-scope-sharing.sql"
    ));
    for statement in parse_postgres_sql(sql).unwrap() {
        let Statement::Query(query) = statement else {
            panic!("expected query")
        };
        let mut outer = Scope::default();
        let mut visible = query::with_scope(&query, &outer, None).into_owned();
        let original = visible.get("scoped").unwrap() as *const SqlBoundQuery;
        let snapshot = visible.clone();
        assert!(Rc::ptr_eq(&visible.frame, &snapshot.frame));
        let mut inner = visible.child();
        assert_eq!(inner.get("scoped").unwrap() as *const _, original);
        inner.insert("scoped".into(), query::sized_by_itself((1, 1)));
        assert_ne!(inner.get("scoped").unwrap() as *const _, original);
        assert_eq!(
            inner.column_names("scoped"),
            Some(&Some(BTreeSet::from(["id".into()])))
        );
        assert_eq!(snapshot.get("scoped").unwrap() as *const _, original);
        // Mutating a cloned frame retains existing bound nodes, not deep copies.
        visible.insert("other".into(), query::sized_by_itself((1, 1)));
        assert_eq!(visible.get("scoped").unwrap() as *const _, original);
        outer.set_columns("scoped".into(), Some(BTreeSet::from(["old".into()])));
        assert_eq!(inner.names()["scoped"], Some(BTreeSet::from(["id".into()])));
        let bound = query::bound_query(&query, &Scope::default(), None);
        let mut tables = Vec::new();
        physical_tables(&bound, &mut tables);
        assert_eq!(tables, ["orders", "accounts"]);
    }
}

#[test]
fn queries_without_with_borrow_the_existing_scope() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/performance/postgres-scopes/independent-32.sql"
    ));
    let statements = parse_postgres_sql(sql).unwrap();
    let Statement::Query(query) = &statements[0] else {
        panic!("expected query")
    };
    let scope = Scope::default();
    let with = query.with.as_ref().unwrap();
    for cte in &with.cte_tables {
        let borrowed = query::with_scope(&cte.query, &scope, None);
        assert!(matches!(borrowed, Cow::Borrowed(_)));
        assert!(std::ptr::eq(borrowed.as_ref(), &scope));
    }
    assert!(scope.get("missing").is_none());
    assert!(scope.column_names("missing").is_none());
}
