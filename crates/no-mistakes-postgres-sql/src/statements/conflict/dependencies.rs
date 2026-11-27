use sqlparser::ast::{Expr, FunctionArg, FunctionArgExpr, FunctionArguments};

/// Only these syntax shapes prove every referenced row column. Unknown AST
/// forms retain a conservative policy result instead of silently dropping keys.
pub(super) fn collect_dependencies(expr: &Expr, columns: &mut Vec<String>) -> bool {
    match expr {
        Expr::Identifier(name) => {
            columns.push(name.value.clone());
            true
        }
        Expr::CompoundIdentifier(names) if !names.is_empty() => {
            columns.push(names.last().unwrap().value.clone());
            true
        }
        Expr::Nested(inner)
        | Expr::UnaryOp { expr: inner, .. }
        | Expr::Cast { expr: inner, .. }
        | Expr::Collate { expr: inner, .. } => collect_dependencies(inner, columns),
        Expr::BinaryOp { left, right, .. } => {
            let left_known = collect_dependencies(left, columns);
            collect_dependencies(right, columns) && left_known
        }
        Expr::Function(function)
            if matches!(function.parameters, FunctionArguments::None)
                && function.within_group.is_empty()
                && function.filter.is_none()
                && function.over.is_none() =>
        {
            let FunctionArguments::List(args) = &function.args else {
                return false;
            };
            if args.duplicate_treatment.is_some() || !args.clauses.is_empty() {
                return false;
            }
            let mut known = true;
            for arg in &args.args {
                let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = arg else {
                    return false;
                };
                known = collect_dependencies(expr, columns) && known;
            }
            known
        }
        Expr::Value(_) => true,
        _ => false,
    }
}
