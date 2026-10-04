use super::super::Resolver;
use super::{
    constructor, fixed_boolean::fixed_scalar_boolean, indexed_base, leaves::numeric_literal, scalar,
};
use crate::codebase::postgres::idents::{ident_key, object_name_ident, unwrap_expr};
use sqlparser::ast::{DataType, Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments};

mod wrappers;

pub(super) fn collect(
    expr: &Expr,
    resolver: &Resolver,
    mut out: Option<&mut Vec<(usize, String)>>,
    indexed: &mut Vec<(usize, String)>,
    types: &mut Vec<String>,
    caller_only: bool,
    positions: super::super::super::super::value::PlaceholderPositions<'_>,
) -> Option<()> {
    match unwrap_expr(expr) {
        Expr::Cast { expr, .. } if out.is_none() => {
            collect(expr, resolver, None, indexed, types, caller_only, positions)
        }
        Expr::CompoundFieldAccess { root, access_chain } if out.is_none() => wrappers::collect(
            root,
            access_chain,
            resolver,
            indexed,
            types,
            caller_only,
            positions,
        ),
        Expr::Value(_) | Expr::Interval(_) => Some(()),
        expr if fixed_scalar_boolean(expr) => Some(()),
        Expr::TypedString(literal) if !matches!(literal.data_type, DataType::Array(_)) => {
            // Parser-custom names such as XML still need catalog scalar evidence.
            if matches!(literal.data_type, DataType::Custom(_, _)) {
                types.push(literal.data_type.to_string());
            }
            Some(())
        }
        Expr::UnaryOp { .. } if numeric_literal(expr) => Some(()),
        Expr::Identifier(ident)
            if super::super::super::super::value::is_placeholder_ident_at(ident, positions) =>
        {
            Some(())
        }
        Expr::Identifier(_) | Expr::CompoundIdentifier(_) => {
            if caller_only {
                return None;
            }
            let column = resolver.column(expr)?;
            if let Some(out) = out {
                out.push(column);
            }
            Some(())
        }
        Expr::CompoundFieldAccess { root, access_chain } if !caller_only => {
            let (base, _) = indexed_base(root, access_chain)?;
            indexed.push(resolver.column(&base)?);
            Some(())
        }
        Expr::Cast {
            expr,
            data_type: DataType::Array(_),
            ..
        } => {
            // A scalar text column can decode an arbitrarily large array.
            if let Some(array) = constructor(expr) {
                for element in &array.elem {
                    collect(
                        element,
                        resolver,
                        out.as_deref_mut(),
                        indexed,
                        types,
                        caller_only,
                        positions,
                    )?;
                }
            } else {
                collect(
                    expr,
                    resolver,
                    out.as_deref_mut(),
                    indexed,
                    types,
                    true,
                    positions,
                )?;
            }
            Some(())
        }
        Expr::Cast {
            expr,
            data_type: data_type @ DataType::Custom(_, _),
            ..
        } => {
            types.push(data_type.to_string());
            if caller_only {
                collect(
                    expr,
                    resolver,
                    out.as_deref_mut(),
                    indexed,
                    types,
                    true,
                    positions,
                )?;
            }
            Some(())
        }
        Expr::Cast {
            expr, data_type, ..
        } if !matches!(data_type, DataType::Array(_) | DataType::Custom(_, _)) => {
            if caller_only {
                collect(
                    expr,
                    resolver,
                    out.as_deref_mut(),
                    indexed,
                    types,
                    true,
                    positions,
                )?;
            }
            Some(())
        }
        Expr::Function(function) => function_columns(
            function,
            resolver,
            out,
            indexed,
            types,
            caller_only,
            positions,
        ),
        Expr::Array(array) => {
            for element in &array.elem {
                collect(
                    element,
                    resolver,
                    out.as_deref_mut(),
                    indexed,
                    types,
                    caller_only,
                    positions,
                )?;
            }
            Some(())
        }
        _ => None,
    }
}

// COALESCE-like syntax can retain array-valued leaves; trusted scalar reducers cannot.
fn function_columns(
    function: &Function,
    resolver: &Resolver,
    mut out: Option<&mut Vec<(usize, String)>>,
    indexed: &mut Vec<(usize, String)>,
    types: &mut Vec<String>,
    caller_only: bool,
    positions: super::super::super::super::value::PlaceholderPositions<'_>,
) -> Option<()> {
    if function.name.0.len() == 1
        && object_name_ident(&function.name).is_some_and(|name| {
            name.quote_style.is_none()
                && ["coalesce", "least", "greatest"].contains(&ident_key(name).as_str())
        })
    {
        let FunctionArguments::List(arguments) = &function.args else {
            return None;
        };
        for argument in &arguments.args {
            let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = argument else {
                return None;
            };
            collect(
                expr,
                resolver,
                out.as_deref_mut(),
                indexed,
                types,
                caller_only,
                positions,
            )?;
        }
    } else {
        // Result scalarity does not remove any argument's row dependency.
        for expr in scalar::arguments(function)? {
            collect(expr, resolver, None, indexed, types, caller_only, positions)?;
        }
    }
    Some(())
}
