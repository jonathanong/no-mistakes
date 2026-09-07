use sqlparser::ast::{BinaryOperator, Expr, Ident, JoinConstraint, JoinOperator};

fn ident_eq() -> Expr {
    Expr::BinaryOp {
        left: Box::new(Expr::Identifier(Ident::new("id"))),
        op: BinaryOperator::Eq,
        right: Box::new(Expr::Identifier(Ident::new("id"))),
    }
}

#[test]
fn join_expr_reads_outer_on_constraints() {
    let expr = ident_eq();
    for operator in [
        JoinOperator::Join(JoinConstraint::On(expr.clone())),
        JoinOperator::Inner(JoinConstraint::On(expr.clone())),
        JoinOperator::Left(JoinConstraint::On(expr.clone())),
        JoinOperator::LeftOuter(JoinConstraint::On(expr.clone())),
        JoinOperator::Right(JoinConstraint::On(expr.clone())),
        JoinOperator::RightOuter(JoinConstraint::On(expr.clone())),
        JoinOperator::FullOuter(JoinConstraint::On(expr.clone())),
    ] {
        assert!(super::join_expr(&operator).is_some());
    }
    assert!(super::join_expr(&JoinOperator::CrossJoin(JoinConstraint::None)).is_none());
}

#[test]
fn left_join_on_predicates_are_collected() {
    let facts = crate::codebase::postgres::statements::extract_sql_statement_facts(
        "SELECT posts.id FROM posts LEFT JOIN topics ON topics.id = posts.topic_id \
         AND topics.parent_id IS NOT NULL",
    );
    let select = facts
        .selects
        .iter()
        .find(|select| select.tables.iter().any(|table| table == "topics"))
        .expect("topics join");
    assert!(
        select
            .predicate_sql
            .to_ascii_lowercase()
            .contains("parent_id is not null"),
        "{select:?}"
    );
}
