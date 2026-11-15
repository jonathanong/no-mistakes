//! Literal values a partial-index predicate can share with a conjunct.
use super::casts::preserves;
use super::columns::cast_fits;
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::SchemaCatalog;
use sqlparser::ast::{ArrayElemTypeDef, DataType, Expr, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in super::super) enum Lit {
    Str(String),
    Bool(bool),
    Num(String),
}

pub(super) fn literal(
    expr: &Expr,
    catalog: &SchemaCatalog,
    column: Option<&str>,
    types: &BTreeMap<String, String>,
) -> Option<Lit> {
    match unwrap_expr(expr) {
        Expr::Value(value) => match &value.value {
            Value::SingleQuotedString(text) => Some(Lit::Str(text.clone())),
            Value::Boolean(flag) => Some(Lit::Bool(*flag)),
            Value::Number(text, _) => Some(Lit::Num(text.clone())),
            _ => None,
        },
        Expr::Cast {
            expr, data_type, ..
        } => {
            let inner = literal(expr, catalog, column, types)?;
            (preserves(data_type, &inner, catalog) && cast_fits(data_type, column, types))
                .then_some(inner)
        }
        _ => None,
    }
}

pub(super) fn literals(
    list: &[Expr],
    catalog: &SchemaCatalog,
    column: Option<&str>,
    types: &BTreeMap<String, String>,
) -> Option<BTreeSet<Lit>> {
    list.iter()
        .map(|expr| literal(expr, catalog, column, types))
        .collect()
}

pub(super) fn array_literals(
    expr: &Expr,
    catalog: &SchemaCatalog,
    column: Option<&str>,
    types: &BTreeMap<String, String>,
) -> Option<BTreeSet<Lit>> {
    match unwrap_expr(expr) {
        Expr::Array(array) => literals(&array.elem, catalog, column, types),
        Expr::Cast {
            expr, data_type, ..
        } => {
            let values = array_literals(expr, catalog, column, types)?;
            let element = array_element(data_type)?;
            values
                .iter()
                .all(|value| {
                    preserves(element, value, catalog) && cast_fits(element, column, types)
                })
                .then_some(values)
        }
        _ => None,
    }
}

fn array_element(data_type: &DataType) -> Option<&DataType> {
    match data_type {
        DataType::Array(
            ArrayElemTypeDef::SquareBracket(inner, _) | ArrayElemTypeDef::Qualified(inner, _),
        ) => Some(inner),
        _ => None,
    }
}
