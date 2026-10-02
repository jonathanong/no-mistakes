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
