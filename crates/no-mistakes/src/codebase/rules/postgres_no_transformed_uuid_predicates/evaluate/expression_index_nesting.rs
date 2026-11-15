//! Drops parentheses PostgreSQL precedence makes redundant.
//! Same-precedence grouping stays, so `(a OR b) AND c` does not become `a OR b AND c`.
use sqlparser::ast::{
    CastKind, Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, Ident,
};

const OTHER: u8 = 7;
const UNARY: u8 = 13;
const CAST: u8 = 15;
const PRIMARY: u8 = 17;

pub(super) fn strip_redundant_nesting(expression: &mut Expr) {
    delimit(expression);
}

fn delimit(expression: &mut Expr) {
    walk(expression);
    while matches!(expression, Expr::Nested(_)) {
        unwrap_nested(expression);
    }
}

fn tighten(expression: &mut Expr, parent: u8) {
    walk(expression);
    while nested_child_precedence(expression).is_some_and(|child| child > parent) {
        unwrap_nested(expression);
    }
}

fn nested_child_precedence(expression: &Expr) -> Option<u8> {
    match expression {
        Expr::Nested(inner) => Some(precedence(inner)),
        _ => None,
    }
}

fn unwrap_nested(expression: &mut Expr) {
    let current = std::mem::replace(expression, Expr::Identifier(Ident::new("_")));
    *expression = match current {
        Expr::Nested(inner) => *inner,
        other => other,
    };
}

fn walk(expression: &mut Expr) {
    match expression {
        Expr::Nested(inner) => walk(inner),
        Expr::BinaryOp { left, right, .. } => {
            tighten(left, OTHER);
            tighten(right, OTHER);
        }
        Expr::UnaryOp { expr, .. } => tighten(expr, UNARY),
        Expr::Cast { kind, expr, .. } => match kind {
            CastKind::DoubleColon => tighten(expr, CAST),
            _ => delimit(expr),
        },
        Expr::Function(function) => walk_function(function),
        Expr::Tuple(items) => items.iter_mut().for_each(delimit),
        _ => {}
    }
}

fn walk_function(function: &mut Function) {
    for arguments in [&mut function.parameters, &mut function.args] {
        let FunctionArguments::List(list) = arguments else {
            continue;
        };
        for argument in &mut list.args {
            if let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = argument {
                delimit(expr);
            }
        }
    }
}

fn precedence(expression: &Expr) -> u8 {
    match expression {
        Expr::BinaryOp { .. } => OTHER,
        Expr::UnaryOp { .. } => UNARY,
        Expr::Cast {
            kind: CastKind::DoubleColon,
            ..
        } => CAST,
        _ => PRIMARY,
    }
}
