use crate::codebase::postgres::idents::unwrap_expr;
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
        } => matches!(
            unwrap_expr(expr.as_ref()),
            Expr::Exists { negated: false, .. }
        ),
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
    let text = lower.as_str();
    let bytes = lower.as_bytes();
    let mut depth = 0i32;
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {
                if depth == 0 && text.is_char_boundary(index) && match_guard(text, index) {
                    return true;
                }
            }
        }
        index += 1;
    }
    false
}

fn match_guard(text: &str, index: usize) -> bool {
    if !token_start(text, index) {
        return false;
    }
    let rest = &text[index..];
    ["where", "and"].into_iter().any(|first| {
        if !rest.starts_with(first) || !ident_boundary(rest, first.len()) {
            return false;
        }
        let Some(after_first) = skip_sql_space(&rest[first.len()..]) else {
            return false;
        };
        if !after_first.starts_with("not") || !ident_boundary(after_first, 3) {
            return false;
        }
        let Some(after_not) = skip_sql_space(&after_first[3..]) else {
            return false;
        };
        after_not.starts_with("exists") && ident_boundary(after_not, 6)
    })
}

fn skip_sql_space(text: &str) -> Option<&str> {
    let trimmed = text.trim_start_matches(|c: char| c.is_ascii_whitespace());
    (trimmed.len() < text.len()).then_some(trimmed)
}

fn token_start(text: &str, index: usize) -> bool {
    index == 0 || ident_boundary(text, index - 1)
}

fn ident_boundary(text: &str, end: usize) -> bool {
    let Some(&next) = text.as_bytes().get(end) else {
        return true;
    };
    !next.is_ascii_alphanumeric() && next != b'_' && next != b'$' && next < 0x80
}
