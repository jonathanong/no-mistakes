//! Shapes a partial-index predicate can share with a query conjunct.
use super::columns::stored_column;
use super::literals::{array_literals, literal, literals};
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::SchemaCatalog;
use sqlparser::ast::{BinaryOperator, Expr, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in super::super) enum Atom {
    Null(String),
    Eq(String, Lit),
    In(String, BTreeSet<Lit>),
}

pub(super) use super::literals::Lit;

pub(super) fn same(have: &Atom, need: &Atom) -> bool {
    match (have, need) {
        (Atom::Null(left), Atom::Null(right)) => left == right,
        (Atom::Eq(left, have), Atom::Eq(right, need)) => left == right && have == need,
        (Atom::In(left, have), Atom::In(right, need)) => left == right && have == need,
        _ => false,
    }
}

pub(super) fn atom_with(
    expr: &Expr,
    catalog: &SchemaCatalog,
    types: &BTreeMap<String, String>,
) -> Option<Atom> {
    if let Some((column, literal)) = eq_literal(expr, catalog, types) {
        return Some(Atom::Eq(column, literal));
    }
    if let Expr::IsNull(column) = unwrap_expr(expr) {
        return plain_column(column).map(Atom::Null);
    }
    membership_with(expr, catalog, types)
}

pub(super) fn membership_with(
    expr: &Expr,
    catalog: &SchemaCatalog,
    types: &BTreeMap<String, String>,
) -> Option<Atom> {
    match unwrap_expr(expr) {
        Expr::InList {
            expr,
            list,
            negated: false,
        } => {
            let column = stored_column(expr, types)?;
            Some(Atom::In(
                column.clone(),
                literals(list, catalog, Some(&column), types)?,
            ))
        }
        Expr::AnyOp {
            left,
            compare_op: BinaryOperator::Eq,
            right,
            ..
        } => {
            let column = stored_column(left, types)?;
            Some(Atom::In(
                column.clone(),
                array_literals(right, catalog, Some(&column), types)?,
            ))
        }
        _ => None,
    }
}

pub(super) fn eq_literal(
    expr: &Expr,
    catalog: &SchemaCatalog,
    types: &BTreeMap<String, String>,
) -> Option<(String, Lit)> {
    let (left, right) = eq_pair(expr)?;
    let (column, value) = column_and_other(left, right, types)?;
    Some((
        column.clone(),
        literal(value, catalog, Some(&column), types)?,
    ))
}

pub(super) fn eq_placeholder(
    expr: &Expr,
    types: &BTreeMap<String, String>,
) -> Option<(String, String)> {
    let (left, right) = eq_pair(expr)?;
    let (column, value) = column_and_other(left, right, types)?;
    Some((column, placeholder(value)?))
}

pub(super) fn binding(expr: &Expr, catalog: &SchemaCatalog) -> Option<(String, Lit)> {
    let types = BTreeMap::new();
    let (left, right) = eq_pair(expr)?;
    if let (Some(name), Some(value)) = (placeholder(left), literal(right, catalog, None, &types)) {
        return Some((name, value));
    }
    if let (Some(name), Some(value)) = (placeholder(right), literal(left, catalog, None, &types)) {
        return Some((name, value));
    }
    None
}

fn column_and_other<'a>(
    left: &'a Expr,
    right: &'a Expr,
    types: &BTreeMap<String, String>,
) -> Option<(String, &'a Expr)> {
    match (stored_column(left, types), stored_column(right, types)) {
        (Some(column), None) => Some((column, right)),
        (None, Some(column)) => Some((column, left)),
        _ => None,
    }
}

fn eq_pair(expr: &Expr) -> Option<(&Expr, &Expr)> {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } => Some((left, right)),
        _ => None,
    }
}

pub(super) fn plain_column(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => Some(ident_key(ident)),
        _ => None,
    }
}

fn placeholder(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Value(value) => match &value.value {
            Value::Placeholder(name) => Some(name.clone()),
            _ => None,
        },
        _ => None,
    }
}
