use super::*;
use sqlparser::ast::{
    FromTable, MergeAction, MergeInsertKind, MergeUpdateKind, OnConflictAction, OnInsert,
    OutputClause, UpdateTableFromKind,
};

fn seeds() -> Vec<Statement> {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres/function-call-clauses/mutations.sql"
    ));
    crate::codebase::postgres::parse::parse_postgres_sql(source).unwrap()
}

fn calls(statement: &Statement) -> Vec<SqlFunctionCallFact> {
    let mut calls = Vec::new();
    collect(statement, &mut calls);
    calls
}

#[test]
fn mutation_clauses_handle_optional_predicates_and_borrow_the_original_ast() {
    let statements = seeds();
    for statement in &statements {
        for call in calls(statement) {
            let expected = match call.name_parts[0].as_str() {
                "probe_values" => SqlFunctionClause::Values,
                "probe_set" => SqlFunctionClause::Set,
                "probe_from" | "probe_using" | "probe_on" => SqlFunctionClause::JoinOn,
                "probe_default" => SqlFunctionClause::Default,
                "probe_when" => SqlFunctionClause::Where,
                "probe_returning" => SqlFunctionClause::Returning,
                name => panic!("unexpected {name}"),
            };
            assert_eq!(call.clause, Some(expected));
        }
    }
    // Public ASTs include alternate dialect spellings; they retain syntactic clauses.
    let mut insert = statements[0].clone();
    let Statement::Insert(value) = &mut insert else {
        panic!()
    };
    let Some(OnInsert::OnConflict(conflict)) = &value.on else {
        panic!()
    };
    let OnConflictAction::DoUpdate(update) = &conflict.action else {
        panic!()
    };
    value.on = Some(OnInsert::DuplicateKeyUpdate(update.assignments.clone()));
    assert!(calls(&insert).iter().any(
        |call| call.name_parts[0] == "probe_set" && call.clause == Some(SqlFunctionClause::Set)
    ));

    let mut update = statements[3].clone();
    let Statement::Update(value) = &mut update else {
        panic!()
    };
    let Some(UpdateTableFromKind::AfterSet(from)) = value.from.take() else {
        panic!()
    };
    value.from = Some(UpdateTableFromKind::BeforeSet(from));
    assert!(calls(&update)
        .iter()
        .any(|call| call.name_parts[0] == "probe_from"
            && call.clause == Some(SqlFunctionClause::JoinOn)));

    let mut delete = statements[4].clone();
    let Statement::Delete(value) = &mut delete else {
        panic!()
    };
    let FromTable::WithFromKeyword(from) = &value.from else {
        panic!()
    };
    value.from = FromTable::WithoutKeyword(from.clone());
    assert!(calls(&delete)
        .iter()
        .any(|call| call.name_parts[0] == "probe_using"
            && call.clause == Some(SqlFunctionClause::JoinOn)));
}

#[test]
fn merge_predicates_and_foreign_output_do_not_invent_returning_membership() {
    let mut statement = seeds().pop().unwrap();
    let Statement::Merge(merge) = &mut statement else {
        panic!()
    };
    let predicate = merge.clauses[0].predicate.clone().unwrap();
    let MergeAction::Update(update) = &mut merge.clauses[0].action else {
        panic!()
    };
    update.update_predicate = Some(predicate.clone());
    update.delete_predicate = Some(predicate.clone());
    let MergeAction::Insert(insert) = &mut merge.clauses[1].action else {
        panic!()
    };
    insert.insert_predicate = Some(predicate);
    assert!(calls(&statement)
        .iter()
        .filter(|call| call.name_parts[0] == "probe_when")
        .all(|call| call.clause == Some(SqlFunctionClause::Where)));
    let Statement::Merge(merge) = &mut statement else {
        panic!()
    };
    let MergeAction::Update(update) = &mut merge.clauses[0].action else {
        panic!()
    };
    update.kind = MergeUpdateKind::Wildcard;
    let MergeAction::Insert(insert) = &mut merge.clauses[1].action else {
        panic!()
    };
    insert.kind = MergeInsertKind::Row;
    let Some(OutputClause::Returning {
        returning_token,
        select_items,
    }) = merge.output.take()
    else {
        panic!()
    };
    merge.output = Some(OutputClause::Output {
        output_token: returning_token,
        select_items,
        into_table: None,
    });
    assert!(calls(&statement)
        .iter()
        .any(|call| call.name_parts[0] == "probe_returning" && call.clause.is_none()));
}
