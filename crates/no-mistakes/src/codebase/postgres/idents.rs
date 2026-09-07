use sqlparser::ast::Expr;

/// Lowercased identifiers referenced anywhere under `expr`.
pub fn collect_ident_names(expr: &Expr) -> Vec<String> {
    let mut names = Vec::new();
    collect_idents(expr, &mut names);
    names.sort();
    names.dedup();
    names
}

pub fn unwrap_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => unwrap_expr(inner),
        other => other,
    }
}

pub(crate) fn visit_child_exprs(expr: &Expr, visit: &mut impl FnMut(&Expr)) {
    match expr {
        Expr::BinaryOp { left, right, .. }
        | Expr::IsDistinctFrom(left, right)
        | Expr::IsNotDistinctFrom(left, right) => {
            visit(left);
            visit(right);
        }
        Expr::UnaryOp { expr, .. }
        | Expr::Cast { expr, .. }
        | Expr::Nested(expr)
        | Expr::IsNull(expr)
        | Expr::IsNotNull(expr)
        | Expr::IsTrue(expr)
        | Expr::IsFalse(expr) => visit(expr),
        Expr::Function(function) => visit_function_arg_exprs(function, visit),
        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => {
            if let Some(operand) = operand {
                visit(operand);
            }
            for case in conditions {
                visit(&case.condition);
                visit(&case.result);
            }
            if let Some(else_result) = else_result {
                visit(else_result);
            }
        }
        Expr::Between {
            expr, low, high, ..
        } => {
            visit(expr);
            visit(low);
            visit(high);
        }
        Expr::InList { expr, list, .. } => {
            visit(expr);
            for item in list {
                visit(item);
            }
        }
        Expr::Like { expr, pattern, .. }
        | Expr::ILike { expr, pattern, .. }
        | Expr::SimilarTo { expr, pattern, .. }
        | Expr::RLike { expr, pattern, .. } => {
            visit(expr);
            visit(pattern);
        }
        _ => {}
    }
}

pub(crate) fn ident_key(ident: &sqlparser::ast::Ident) -> String {
    if ident.quote_style.is_some() {
        ident.value.clone()
    } else {
        ident.value.to_ascii_lowercase()
    }
}

pub(crate) fn object_name_ident(
    name: &sqlparser::ast::ObjectName,
) -> Option<&sqlparser::ast::Ident> {
    name.0.iter().rev().find_map(|part| match part {
        sqlparser::ast::ObjectNamePart::Identifier(ident) => Some(ident),
        _ => None,
    })
}

fn collect_idents(expr: &Expr, names: &mut Vec<String>) {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => names.push(ident.value.to_ascii_lowercase()),
        Expr::CompoundIdentifier(parts) => {
            if let Some(ident) = parts.last() {
                names.push(ident.value.to_ascii_lowercase());
            }
        }
        other => visit_child_exprs(other, &mut |child| collect_idents(child, names)),
    }
}

fn visit_function_arg_exprs(function: &sqlparser::ast::Function, visit: &mut impl FnMut(&Expr)) {
    let sqlparser::ast::FunctionArguments::List(list) = &function.args else {
        return;
    };
    visit_function_args(&list.args, visit);
}

pub(crate) fn visit_function_args(
    args: &[sqlparser::ast::FunctionArg],
    visit: &mut impl FnMut(&Expr),
) {
    for arg in args {
        if let Some(expr) = function_arg_expr(arg) {
            visit(expr);
        }
    }
}

fn function_arg_expr(arg: &sqlparser::ast::FunctionArg) -> Option<&Expr> {
    match arg {
        sqlparser::ast::FunctionArg::Unnamed(sqlparser::ast::FunctionArgExpr::Expr(expr))
        | sqlparser::ast::FunctionArg::Named {
            arg: sqlparser::ast::FunctionArgExpr::Expr(expr),
            ..
        }
        | sqlparser::ast::FunctionArg::ExprNamed {
            arg: sqlparser::ast::FunctionArgExpr::Expr(expr),
            ..
        } => Some(expr),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
