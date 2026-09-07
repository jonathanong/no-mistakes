use sqlparser::ast::{BinaryOperator, Expr, Query, SetExpr, UnaryOperator};

pub(super) fn query_is_guarded(query: &Query) -> bool {
    set_expr_is_guarded(&query.body)
}

fn set_expr_is_guarded(expr: &SetExpr) -> bool {
    match expr {
        SetExpr::Select(select) => select
            .selection
            .as_ref()
            .is_some_and(has_conjunctive_not_exists),
        SetExpr::Query(query) => query_is_guarded(query),
        SetExpr::SetOperation { left, right, .. } => {
            set_expr_is_guarded(left) && set_expr_is_guarded(right)
        }
        _ => false,
    }
}

pub(super) fn has_conjunctive_not_exists(expr: &Expr) -> bool {
    match expr {
        Expr::Nested(inner) => has_conjunctive_not_exists(inner),
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr,
        } => matches!(expr.as_ref(), Expr::Exists { negated: false, .. }),
        Expr::Exists { negated: true, .. } => true,
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => has_conjunctive_not_exists(left) || has_conjunctive_not_exists(right),
        _ => false,
    }
}

/// Quote-masked `WHERE|AND NOT EXISTS` at paren-depth zero.
pub fn has_top_level_conjunctive_not_exists(masked: &str) -> bool {
    let lower = masked.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut depth = 0i32;
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {
                if depth == 0 && match_guard(&lower, index) {
                    return true;
                }
            }
        }
        index += 1;
    }
    false
}

fn match_guard(text: &str, index: usize) -> bool {
    let rest = &text[index..];
    rest.starts_with("where not exists") || rest.starts_with("and not exists")
}
