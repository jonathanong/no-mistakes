use sqlparser::ast::{
    CaseWhen, CastKind, Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, Ident,
    InTarget, UnaryOperator,
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
    while peel_outer(expression) {}
}

fn tighten(expression: &mut Expr, parent: u8) {
    walk(expression);
    while nested_child_precedence(expression).is_some_and(|child| child > parent) {
        peel_outer(expression);
    }
}

fn nested_child_precedence(expression: &Expr) -> Option<u8> {
    match expression {
        Expr::Nested(inner) => Some(precedence(inner)),
        _ => None,
    }
}

fn peel_outer(expression: &mut Expr) -> bool {
    let Expr::Nested(inner) = expression else {
        return false;
    };
    let owned = std::mem::replace(inner.as_mut(), Expr::Identifier(Ident::new("_")));
    *expression = owned;
    true
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
        Expr::IsNull(expr)
        | Expr::IsNotNull(expr)
        | Expr::IsTrue(expr)
        | Expr::IsNotTrue(expr)
        | Expr::IsFalse(expr)
        | Expr::IsNotFalse(expr)
        | Expr::IsUnknown(expr)
        | Expr::IsNotUnknown(expr) => tighten(expr, OTHER),
        Expr::IsDistinctFrom(left, right) | Expr::IsNotDistinctFrom(left, right) => {
            tighten(left, OTHER);
            tighten(right, OTHER);
        }
        Expr::Between {
            expr, low, high, ..
        } => {
            tighten(expr, OTHER);
            tighten(low, OTHER);
            tighten(high, OTHER);
        }
        Expr::In {
            expr,
            target: InTarget::List(list),
            ..
        } => {
            tighten(expr, OTHER);
            list.iter_mut().for_each(delimit);
        }
        Expr::Collate { expr, .. } => tighten(expr, OTHER),
        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => walk_case(operand, conditions, else_result),
        _ => {}
    }
}

fn walk_case(
    operand: &mut Option<Box<Expr>>,
    conditions: &mut [CaseWhen],
    else_result: &mut Option<Box<Expr>>,
) {
    if let Some(operand) = operand {
        delimit(operand);
    }
    for when in conditions {
        delimit(&mut when.condition);
        delimit(&mut when.result);
    }
    if let Some(result) = else_result {
        delimit(result);
    }
}

fn walk_function(function: &mut Function) {
    for arguments in [&mut function.parameters, &mut function.args] {
        let FunctionArguments::List(list) = arguments else {
            continue;
        };
        for argument in &mut list.args {
            if let Some(expr) = argument_expr(argument) {
                delimit(expr);
            }
        }
    }
}

fn argument_expr(argument: &mut FunctionArg) -> Option<&mut Expr> {
    match argument {
        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr))
        | FunctionArg::Named {
            arg: FunctionArgExpr::Expr(expr),
            ..
        }
        | FunctionArg::ExprNamed {
            arg: FunctionArgExpr::Expr(expr),
            ..
        } => Some(expr),
        _ => None,
    }
}

fn precedence(expression: &Expr) -> u8 {
    match expression {
        Expr::BinaryOp { .. } => OTHER,
        Expr::UnaryOp {
            op: UnaryOperator::Plus | UnaryOperator::Minus,
            ..
        } => UNARY,
        Expr::Cast { .. } => CAST,
        expr if primary(expr) => PRIMARY,
        _ => OTHER,
    }
}

fn primary(expression: &Expr) -> bool {
    matches!(
        expression,
        Expr::Identifier(_)
            | Expr::CompoundIdentifier(_)
            | Expr::Value(_)
            | Expr::TypedString(_)
            | Expr::Function(_)
            | Expr::Nested(_)
            | Expr::Tuple(_)
            | Expr::Case { .. }
    )
}
