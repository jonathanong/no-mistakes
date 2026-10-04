use super::conjuncts::placeholders::is_recovered_placeholder;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use sqlparser::ast::{DataType, Expr, Statement, Value};

/// Declared bind types belong to the enclosing statement, not to unrelated parameters.
#[derive(Default)]
pub(super) struct Int4Bindings(Option<Vec<bool>>);

impl Int4Bindings {
    pub(super) fn for_statement(statement: &Statement) -> Self {
        match statement {
            Statement::Prepare {
                data_types,
                statement,
                ..
            } => {
                if data_types.is_empty() {
                    Self::for_statement(statement)
                } else {
                    Self(Some(data_types.iter().map(is_int4).collect()))
                }
            }
            Statement::Explain { statement, .. } => Self::for_statement(statement),
            _ => Self::default(),
        }
    }

    fn allows(&self, name: Option<&str>) -> bool {
        let Some(types) = &self.0 else {
            return true;
        };
        name.and_then(|name| name.strip_prefix('$'))
            .and_then(|index| index.parse::<usize>().ok())
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| types.get(index))
            .copied()
            .unwrap_or(false)
    }
}

fn is_int4(data_type: &DataType) -> bool {
    match data_type {
        DataType::Int(_) | DataType::Int4(_) | DataType::Integer(_) => true,
        DataType::Custom(name, modifiers) if modifiers.is_empty() => {
            let [schema, name] = name.0.as_slice() else {
                return false;
            };
            schema
                .as_ident()
                .is_some_and(|ident| ident_key(ident) == "pg_catalog")
                && name
                    .as_ident()
                    .is_some_and(|ident| ident_key(ident) == "int4")
        }
        _ => false,
    }
}

/// Reuse AST leaves, normalizing tuple elements without rewriting or reparsing source SQL.
#[derive(PartialEq)]
pub(super) enum Identity<'a> {
    Expression(&'a Expr, Option<&'a str>),
    Tuple(Vec<Identity<'a>>),
}

pub(super) fn of<'a>(
    expr: &'a Expr,
    bindings: &Int4Bindings,
    positions: &[(u32, u32)],
) -> Option<Identity<'a>> {
    let expr = unwrap_expr(expr);
    match expr {
        Expr::Value(value) => {
            let Value::Placeholder(name) = &value.value else {
                return None;
            };
            Some(Identity::Expression(expr, Some(name)))
        }
        Expr::Identifier(ident) if is_recovered_placeholder(ident, positions) => {
            Some(Identity::Expression(expr, None))
        }
        Expr::Tuple(items) if !items.is_empty() => items
            .iter()
            .map(|item| of(item, bindings, positions))
            .collect::<Option<Vec<_>>>()
            .map(Identity::Tuple),
        Expr::Cast {
            expr: inner,
            data_type,
            ..
        } => {
            let identity = of(inner, bindings, positions)?;
            if is_int4(data_type) {
                if let Identity::Expression(inner, parameter) = &identity {
                    // An opaque inner cast can change the value; an outer int4 cast cannot erase it.
                    if !matches!(inner, Expr::Cast { .. }) && bindings.allows(*parameter) {
                        return Some(identity);
                    }
                }
            }
            Some(Identity::Expression(expr, None))
        }
        _ => None,
    }
}
