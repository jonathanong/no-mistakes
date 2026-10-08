use super::super::expressions::name;
use super::super::types::*;
use sqlparser::ast::{Expr, ObjectName};

pub(super) fn root(expr: &Expr) -> PostgresSqlExpressionChildRoot {
    use PostgresSqlExpressionChildRoot as Root;
    match expr {
        Expr::Identifier(ident) => Root::ColumnReference {
            name: name(&ObjectName::from(vec![ident.clone()])),
        },
        Expr::CompoundIdentifier(idents) => Root::ColumnReference {
            name: name(&ObjectName::from(idents.clone())),
        },
        Expr::Function(function) => {
            let (syntax, arguments_complete, mut modifiers) = match &function.args {
                sqlparser::ast::FunctionArguments::None => {
                    (PostgresSqlFunctionSyntax::Value, true, Vec::new())
                }
                sqlparser::ast::FunctionArguments::Subquery(_) => {
                    (PostgresSqlFunctionSyntax::Call, false, Vec::new())
                }
                sqlparser::ast::FunctionArguments::List(list) => {
                    let complete = list.args.iter().all(|argument| match argument {
                        sqlparser::ast::FunctionArg::Unnamed(value)
                        | sqlparser::ast::FunctionArg::Named { arg: value, .. } => {
                            matches!(value, sqlparser::ast::FunctionArgExpr::Expr(_))
                        }
                        sqlparser::ast::FunctionArg::ExprNamed { name, arg, .. } => {
                            matches!(name, Expr::Identifier(_))
                                && matches!(arg, sqlparser::ast::FunctionArgExpr::Expr(_))
                        }
                    });
                    let mut modifiers = Vec::new();
                    if let Some(treatment) = &list.duplicate_treatment {
                        modifiers.push(treatment.to_string());
                    }
                    modifiers.extend(list.clauses.iter().map(ToString::to_string));
                    (PostgresSqlFunctionSyntax::Call, complete, modifiers)
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
                    "WITHIN GROUP (ORDER BY {})",
                    function
                        .within_group
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            Root::FunctionCall {
                name: name(&function.name),
                arguments_complete,
                syntax,
                modifiers,
            }
        }
        Expr::Nested(_) => Root::Parenthesized,
        Expr::Cast {
            kind, data_type, ..
        } => Root::Cast {
            cast_kind: format!("{kind:?}").to_ascii_lowercase(),
            data_type: data_type.to_string(),
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
        Expr::Subquery(_) | Expr::Exists { .. } => Root::Subquery,
        _ => Root::Other,
    }
}
