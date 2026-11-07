use sqlparser::ast::{BinaryOperator as Op, Expr, UnaryOperator};

pub(super) fn boolean_result(expr: &Expr) -> bool {
    match expr {
        Expr::BinaryOp { op, .. } => matches!(
            op,
            Op::Eq
                | Op::NotEq
                | Op::Gt
                | Op::Lt
                | Op::GtEq
                | Op::LtEq
                | Op::Spaceship
                | Op::And
                | Op::Or
                | Op::Xor
                | Op::PGOverlap
                | Op::PGRegexMatch
                | Op::PGRegexIMatch
                | Op::PGRegexNotMatch
                | Op::PGRegexNotIMatch
                | Op::PGLikeMatch
                | Op::PGILikeMatch
                | Op::PGNotLikeMatch
                | Op::PGNotILikeMatch
                | Op::PGStartsWith
                | Op::AtAt
                | Op::AtArrow
                | Op::ArrowAt
                | Op::AtQuestion
                | Op::Question
                | Op::QuestionAnd
                | Op::QuestionPipe
                | Op::Overlaps
        ),
        Expr::UnaryOp { op, .. } => matches!(op, UnaryOperator::Not | UnaryOperator::BangNot),
        _ => matches!(
            expr,
            Expr::IsFalse(_)
                | Expr::IsNotFalse(_)
                | Expr::IsTrue(_)
                | Expr::IsNotTrue(_)
                | Expr::IsNull(_)
                | Expr::IsNotNull(_)
                | Expr::IsUnknown(_)
                | Expr::IsNotUnknown(_)
                | Expr::IsDistinctFrom(..)
                | Expr::IsNotDistinctFrom(..)
                | Expr::IsJson { .. }
                | Expr::IsNormalized { .. }
                | Expr::InList { .. }
                | Expr::InSubquery { .. }
                | Expr::InUnnest { .. }
                | Expr::Between { .. }
                | Expr::Like { .. }
                | Expr::ILike { .. }
                | Expr::SimilarTo { .. }
                | Expr::RLike { .. }
                | Expr::AnyOp { .. }
                | Expr::AllOp { .. }
                | Expr::Exists { .. }
        ),
    }
}
