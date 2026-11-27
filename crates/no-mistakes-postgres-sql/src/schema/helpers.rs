use sqlparser::ast::{
    Expr, FunctionArg, FunctionArgExpr, FunctionArguments, IndexColumn, ObjectName, ObjectNamePart,
};

fn unwrap_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => unwrap_expr(inner),
        other => other,
    }
}

/// Function name of a generated-column expression, if the root is a call.
pub fn generated_function(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Function(function) => {
            let name = relation_name(&function.name);
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        }
        _ => None,
    }
}

/// Identifier arguments of a generated-column function, lowercased.
pub fn generated_function_arg_columns(expr: &Expr) -> Vec<String> {
    let Expr::Function(function) = unwrap_expr(expr) else {
        return Vec::new();
    };
    let FunctionArguments::List(list) = &function.args else {
        return Vec::new();
    };
    list.args
        .iter()
        .filter_map(|arg| {
            let expr = match arg {
                FunctionArg::Unnamed(FunctionArgExpr::Expr(expr))
                | FunctionArg::Named {
                    arg: FunctionArgExpr::Expr(expr),
                    ..
                }
                | FunctionArg::ExprNamed {
                    arg: FunctionArgExpr::Expr(expr),
                    ..
                } => expr,
                _ => return None,
            };
            expr_column_name(expr).map(|name| name.to_ascii_lowercase())
        })
        .collect()
}

/// Last identifier in an index column expression.
pub fn index_column_name(column: &IndexColumn) -> Option<String> {
    expr_column_name(&column.column.expr)
}

/// Relation name: the last identifier, not a schema qualifier.
pub fn relation_name(name: &ObjectName) -> String {
    name.0
        .iter()
        .rev()
        .find_map(|part| match part {
            ObjectNamePart::Identifier(ident) => Some(ident.value.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

fn expr_column_name(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => Some(ident.value.clone()),
        Expr::CompoundIdentifier(parts) => parts.last().map(|ident| ident.value.clone()),
        _ => None,
    }
}
