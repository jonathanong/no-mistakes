use super::*;
use sqlparser::ast::{JoinConstraint, JoinOperator, SetExpr};

fn seed() -> Statement {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres/function-call-clauses/joins.sql"
    ));
    crate::codebase::postgres::parse::parse_postgres_sql(source)
        .unwrap()
        .pop()
        .unwrap()
}

fn set_operator(statement: &mut Statement, operator: JoinOperator) {
    let Statement::Query(query) = statement else {
        panic!("expected query fixture");
    };
    let SetExpr::Select(select) = query.body.as_mut() else {
        panic!("expected SELECT fixture");
    };
    select.from[0].joins[0].join_operator = operator;
}

#[test]
fn join_constraints_keep_their_clause_across_public_ast_variants() {
    let statement = seed();
    let Statement::Query(query) = &statement else {
        panic!("expected query fixture");
    };
    let SetExpr::Select(select) = query.body.as_ref() else {
        panic!("expected SELECT fixture");
    };
    let JoinOperator::Join(constraint) = &select.from[0].joins[0].join_operator else {
        panic!("expected JOIN fixture");
    };
    // The shared visitor accepts ASTs, so preserve ON identity without inferring
    // equivalent syntax or clause membership for other dialects' join conditions.
    let variants: [fn(JoinConstraint) -> JoinOperator; 15] = [
        JoinOperator::Join,
        JoinOperator::Inner,
        JoinOperator::Left,
        JoinOperator::LeftOuter,
        JoinOperator::Right,
        JoinOperator::RightOuter,
        JoinOperator::FullOuter,
        JoinOperator::CrossJoin,
        JoinOperator::Semi,
        JoinOperator::LeftSemi,
        JoinOperator::RightSemi,
        JoinOperator::Anti,
        JoinOperator::LeftAnti,
        JoinOperator::RightAnti,
        JoinOperator::StraightJoin,
    ];
    for variant in variants {
        let mut changed = statement.clone();
        set_operator(&mut changed, variant(constraint.clone()));
        let mut calls = Vec::new();
        collect(&changed, &mut calls);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].clause, Some(SqlFunctionClause::JoinOn));
    }
    let JoinConstraint::On(expression) = constraint else {
        panic!("expected ON fixture");
    };
    let mut changed = statement.clone();
    set_operator(
        &mut changed,
        JoinOperator::AsOf {
            match_condition: expression.clone(),
            constraint: constraint.clone(),
        },
    );
    let mut calls = Vec::new();
    collect(&changed, &mut calls);
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].clause, None);
    assert_eq!(calls[1].clause, Some(SqlFunctionClause::JoinOn));

    for operator in [
        JoinOperator::Inner(JoinConstraint::Natural),
        JoinOperator::CrossApply,
        JoinOperator::OuterApply,
        JoinOperator::ArrayJoin,
        JoinOperator::LeftArrayJoin,
        JoinOperator::InnerArrayJoin,
    ] {
        let mut changed = statement.clone();
        set_operator(&mut changed, operator);
        let mut calls = Vec::new();
        collect(&changed, &mut calls);
        assert!(calls.is_empty());
    }
}
