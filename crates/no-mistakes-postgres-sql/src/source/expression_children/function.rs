use super::super::expressions::identifier;
use super::super::types::PostgresSqlExpressionChildRole as Role;
use super::super::types::PostgresSqlIdentifier;
use super::Spec;
use sqlparser::ast::{Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments};

pub(super) fn specs(function: &Function) -> (Vec<Spec<'_>>, bool) {
    let mut complete = function.parameters == FunctionArguments::None
        && function.within_group.is_empty()
        && function.over.is_none()
        && function.null_treatment.is_none()
        && function.filter.is_none();
    let mut children = Vec::new();
    match &function.args {
        FunctionArguments::None => {}
        FunctionArguments::Subquery(_) => complete = false,
        FunctionArguments::List(list) => {
            complete &= list.duplicate_treatment.is_none() && list.clauses.is_empty();
            for (index, arg) in list.args.iter().enumerate() {
                let (value, argument_name) = argument(arg, &mut complete);
                if let FunctionArgExpr::Expr(expr) = value {
                    children.push(Spec {
                        role: Role::Argument,
                        index: Some(index),
                        argument_name,
                        expr,
                    });
                } else {
                    complete = false;
                }
            }
        }
    }
    if let Some(filter) = &function.filter {
        children.push(Spec {
            role: Role::FilterPredicate,
            index: None,
            argument_name: None,
            expr: filter,
        });
    }
    (children, complete)
}

fn argument<'a>(
    argument: &'a FunctionArg,
    complete: &mut bool,
) -> (&'a FunctionArgExpr, Option<PostgresSqlIdentifier>) {
    match argument {
        FunctionArg::Unnamed(value) => (value, None),
        FunctionArg::Named { name, arg, .. } => (arg, Some(identifier(name))),
        FunctionArg::ExprNamed { name, arg, .. } => {
            let name = match name {
                Expr::Identifier(ident) => Some(identifier(ident)),
                _ => {
                    *complete = false;
                    None
                }
            };
            (arg, name)
        }
    }
}
