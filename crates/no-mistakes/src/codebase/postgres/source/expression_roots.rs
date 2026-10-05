use super::{
    expressions::{identifier, name},
    locations::Locations,
    types::*,
};
use sqlparser::ast::{
    Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, ObjectName, Spanned,
};

pub(super) fn root(expr: &Expr, locations: &Locations<'_>) -> PostgresSqlExpressionRoot {
    use PostgresSqlExpressionRoot as Root;
    match expr {
        Expr::Identifier(ident) => Root::ColumnReference {
            name: name(&ObjectName::from(vec![ident.clone()])),
        },
        Expr::CompoundIdentifier(idents) => Root::ColumnReference {
            name: name(&ObjectName::from(idents.clone())),
        },
        Expr::Function(function) => function_root(function, locations),
        Expr::Nested(inner) => Root::Parenthesized {
            expression: Box::new(root(inner, locations)),
        },
        Expr::Cast {
            expr, data_type, ..
        } => Root::Cast {
            data_type: data_type.to_string(),
            expression: Box::new(root(expr, locations)),
        },
        Expr::Value(_) | Expr::TypedString { .. } => Root::Literal {
            sql: expr.to_string(),
        },
        Expr::BinaryOp { op, .. } => Root::Binary {
            operator: op.to_string(),
        },
        Expr::UnaryOp { op, .. } => Root::Unary {
            operator: op.to_string(),
        },
        Expr::Case { .. } => Root::Case,
        Expr::Subquery(_) => Root::Subquery,
        _ => Root::Other,
    }
}

fn function_root(function: &Function, locations: &Locations<'_>) -> PostgresSqlExpressionRoot {
    let mut arguments = Vec::new();
    let mut complete = true;
    let mut modifiers = Vec::new();
    let syntax = match &function.args {
        FunctionArguments::None => PostgresSqlFunctionSyntax::Value,
        FunctionArguments::Subquery(_) => {
            complete = false;
            PostgresSqlFunctionSyntax::Call
        }
        FunctionArguments::List(list) => {
            if let Some(treatment) = &list.duplicate_treatment {
                modifiers.push(treatment.to_string());
            }
            modifiers.extend(list.clauses.iter().map(ToString::to_string));
            for arg in &list.args {
                let (argument_name, value) = match arg {
                    FunctionArg::Unnamed(value) => (None, value),
                    FunctionArg::Named { name, arg, .. } => (Some(identifier(name)), arg),
                    FunctionArg::ExprNamed { name, arg, .. } => {
                        let argument_name = if let Expr::Identifier(ident) = name {
                            Some(identifier(ident))
                        } else {
                            complete = false;
                            None
                        };
                        (argument_name, arg)
                    }
                };
                let (argument_root, span) = match value {
                    FunctionArgExpr::Expr(expr) => {
                        (root(expr, locations), locations.span(expr.span()))
                    }
                    _ => {
                        complete = false;
                        (PostgresSqlExpressionRoot::Other, None)
                    }
                };
                arguments.push(PostgresSqlCallArgument {
                    name: argument_name,
                    sql: value.to_string(),
                    span,
                    root: argument_root,
                });
            }
            PostgresSqlFunctionSyntax::Call
        }
    };
    if let Some(filter) = &function.filter {
        modifiers.push(format!("FILTER (WHERE {filter})"));
    }
    if let Some(over) = &function.over {
        modifiers.push(format!("OVER {over}"));
    }
    if let Some(treatment) = &function.null_treatment {
        modifiers.push(treatment.to_string());
    }
    if !function.within_group.is_empty() {
        modifiers.push(format!(
            "WITHIN GROUP ({})",
            function
                .within_group
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    PostgresSqlExpressionRoot::FunctionCall {
        name: name(&function.name),
        arguments,
        arguments_complete: complete,
        syntax,
        modifiers,
    }
}
