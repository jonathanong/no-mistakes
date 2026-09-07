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

fn collect_idents(expr: &Expr, names: &mut Vec<String>) {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => names.push(ident.value.to_ascii_lowercase()),
        Expr::CompoundIdentifier(parts) => {
            if let Some(ident) = parts.last() {
                names.push(ident.value.to_ascii_lowercase());
            }
        }
        other => walk_child_exprs(other, names),
    }
}

fn walk_child_exprs(expr: &Expr, names: &mut Vec<String>) {
    match expr {
        Expr::BinaryOp { left, right, .. } => {
            collect_idents(left, names);
            collect_idents(right, names);
        }
        Expr::UnaryOp { expr, .. } | Expr::Cast { expr, .. } | Expr::Nested(expr) => {
            collect_idents(expr, names);
        }
        Expr::Function(function) => collect_function_idents(function, names),
        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => {
            if let Some(operand) = operand {
                collect_idents(operand, names);
            }
            for case in conditions {
                collect_idents(&case.condition, names);
                collect_idents(&case.result, names);
            }
            if let Some(else_result) = else_result {
                collect_idents(else_result, names);
            }
        }
        Expr::IsNull(inner)
        | Expr::IsNotNull(inner)
        | Expr::IsTrue(inner)
        | Expr::IsFalse(inner) => {
            collect_idents(inner, names);
        }
        Expr::IsDistinctFrom(left, right) | Expr::IsNotDistinctFrom(left, right) => {
            collect_idents(left, names);
            collect_idents(right, names);
        }
        Expr::Between {
            expr, low, high, ..
        } => {
            collect_idents(expr, names);
            collect_idents(low, names);
            collect_idents(high, names);
        }
        Expr::InList { expr, list, .. } => {
            collect_idents(expr, names);
            for item in list {
                collect_idents(item, names);
            }
        }
        _ => {}
    }
}

fn collect_function_idents(function: &sqlparser::ast::Function, names: &mut Vec<String>) {
    let sqlparser::ast::FunctionArguments::List(list) = &function.args else {
        return;
    };
    for arg in &list.args {
        if let Some(expr) = function_arg_expr(arg) {
            collect_idents(expr, names);
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
