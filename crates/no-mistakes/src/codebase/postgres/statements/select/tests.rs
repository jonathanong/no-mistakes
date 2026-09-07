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
        JoinOperator::Inner(JoinConstraint::On(expr.clone())),
        JoinOperator::LeftOuter(JoinConstraint::On(expr.clone())),
        JoinOperator::RightOuter(JoinConstraint::On(expr.clone())),
        JoinOperator::FullOuter(JoinConstraint::On(expr.clone())),
    ] {
        assert!(super::join_expr(&operator).is_some());
    }
    assert!(super::join_expr(&JoinOperator::CrossJoin(JoinConstraint::None)).is_none());
}
