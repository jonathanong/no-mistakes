use super::*;
use sqlparser::ast::{TableFactor, TableObject};

fn statements() -> Vec<Statement> {
    let sql = include_str!("../../../../../../../test-cases/rules/postgres-no-generated-column-writes/unit-fixture/prepared-write-forms/forms.sql");
    crate::codebase::postgres::parse_postgres_sql(sql).unwrap()
}

#[test]
fn write_fact_forms_preserve_named_positional_and_wildcard_targets() {
    let mut statements = statements();
    let mut out = Vec::new();
    for statement in &statements {
        collect(statement, &mut out);
    }
    assert!(out
        .iter()
        .any(|fact| fact.columns == SqlWriteColumns::Positional(Some(2))));
    let Statement::Merge(merge) = &mut statements[2] else {
        panic!("merge")
    };
    let MergeAction::Update(update) = &mut merge.clauses[0].action else {
        panic!("update")
    };
    update.kind = MergeUpdateKind::Wildcard;
    let MergeAction::Insert(insert) = &mut merge.clauses[1].action else {
        panic!("insert")
    };
    insert.columns.clear();
    insert.kind = MergeInsertKind::Wildcard;
    out.clear();
    collect(&statements[2], &mut out);
    assert_eq!(
        out.iter()
            .filter(|fact| fact.columns == SqlWriteColumns::All)
            .count(),
        2
    );
    let Statement::Merge(merge) = &mut statements[2] else {
        panic!("merge")
    };
    let MergeAction::Insert(insert) = &mut merge.clauses[1].action else {
        panic!("insert")
    };
    insert.kind = MergeInsertKind::Row;
    out.clear();
    collect(&statements[2], &mut out);
    assert!(out
        .iter()
        .any(|fact| fact.columns == SqlWriteColumns::Positional(Some(usize::MAX))));
}

#[test]
fn query_targets_are_not_base_relation_write_facts() {
    let mut statements = statements();
    let Statement::Query(query) = statements.pop().unwrap() else {
        panic!("query")
    };
    let Statement::Insert(insert) = &mut statements[0] else {
        panic!("insert")
    };
    insert.table = TableObject::TableQuery(query.clone());
    let derived = TableFactor::Derived {
        lateral: false,
        subquery: query,
        alias: None,
        sample: None,
    };
    let Statement::Update(update) = &mut statements[1] else {
        panic!("update")
    };
    update.table.relation = derived.clone();
    let Statement::Merge(merge) = &mut statements[2] else {
        panic!("merge")
    };
    merge.table = derived;
    let mut out = Vec::new();
    for statement in &statements[..3] {
        collect(statement, &mut out);
    }
    assert!(out.is_empty());
}

#[test]
fn columnless_shapes_record_write_facts_in_source_order() {
    let sql = include_str!("../../../../../../../test-cases/rules/postgres-no-generated-column-writes/unit-fixture/columnless-shapes/writes.sql");
    let statements = crate::codebase::postgres::parse::parse_postgres_sql_lenient(sql);
    let mut out = Vec::new();
    for statement in &statements {
        collect(statement, &mut out);
    }
    let facts: Vec<_> = out
        .iter()
        .map(|fact| (fact.line, fact.table.as_str(), fact.columns.clone()))
        .collect();
    assert_eq!(
        facts,
        vec![
            (1, "votes", SqlWriteColumns::Positional(Some(2))),
            (2, "votes", SqlWriteColumns::Positional(Some(2))),
            (3, "votes", SqlWriteColumns::Positional(Some(usize::MAX))),
            (4, "votes", SqlWriteColumns::Positional(Some(2))),
            (5, "votes", SqlWriteColumns::Positional(Some(2))),
            (6, "votes", SqlWriteColumns::Positional(Some(2))),
            (7, "votes", SqlWriteColumns::Positional(Some(2))),
            (8, "votes", SqlWriteColumns::Positional(None)),
            (9, "items", SqlWriteColumns::Positional(Some(2))),
            (10, "items", SqlWriteColumns::Positional(Some(3))),
            (11, "items", SqlWriteColumns::Positional(Some(usize::MAX))),
            (12, "items", SqlWriteColumns::Positional(None)),
            (13, "votes", SqlWriteColumns::Named(vec!["id".to_string()])),
            (
                14,
                "items",
                SqlWriteColumns::Named(vec!["created_at".to_string()]),
            ),
            (
                15,
                "items",
                SqlWriteColumns::Named(vec!["note".to_string()]),
            ),
            (19, "items", SqlWriteColumns::Positional(Some(3))),
            (20, "votes", SqlWriteColumns::Positional(Some(2))),
            (
                21,
                "items",
                SqlWriteColumns::Named(vec!["id".to_string(), "note".to_string()]),
            ),
            (
                21,
                "items",
                SqlWriteColumns::Named(vec!["created_at".to_string()]),
            ),
            (23, "items", SqlWriteColumns::Positional(Some(3))),
            (28, "items", SqlWriteColumns::Positional(Some(3))),
        ]
    );
}
