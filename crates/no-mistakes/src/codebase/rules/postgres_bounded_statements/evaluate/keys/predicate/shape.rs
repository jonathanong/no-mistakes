//! Shapes a partial-index predicate can share with a query conjunct.
use super::casts::preserves;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::SchemaCatalog;
use sqlparser::ast::{BinaryOperator, Expr, Value};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in super::super) enum Atom {
    Null(String),
    Eq(String, Lit),
    In(String, BTreeSet<Lit>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in super::super) enum Lit {
    Str(String),
    Bool(bool),
    Num(String),
}

pub(super) fn same(have: &Atom, need: &Atom) -> bool {
    match (have, need) {
        (Atom::Null(left), Atom::Null(right)) => left == right,
        (Atom::Eq(left, have), Atom::Eq(right, need)) => left == right && have == need,
        (Atom::In(left, have), Atom::In(right, need)) => left == right && have == need,
        _ => false,
    }
}

pub(super) fn atom(expr: &Expr, catalog: &SchemaCatalog) -> Option<Atom> {
    if let Some((column, literal)) = eq_literal(expr, catalog) {
        return Some(Atom::Eq(column, literal));
    }
    if let Expr::IsNull(column) = unwrap_expr(expr) {
        return plain_column(column).map(Atom::Null);
    }
    membership(expr, catalog)
}

pub(super) fn membership(expr: &Expr, catalog: &SchemaCatalog) -> Option<Atom> {
    match unwrap_expr(expr) {
        Expr::InList {
            expr,
            list,
            negated: false,
        } => Some(Atom::In(plain_column(expr)?, literals(list, catalog)?)),
        Expr::AnyOp {
            left,
            compare_op: BinaryOperator::Eq,
            right,
            ..
        } => Some(Atom::In(
            plain_column(left)?,
            array_literals(right, catalog)?,
        )),
        _ => None,
    }
}

pub(super) fn eq_literal(expr: &Expr, catalog: &SchemaCatalog) -> Option<(String, Lit)> {
    let (left, right) = eq_pair(expr)?;
    let (column, value) = column_and_other(left, right)?;
    Some((column, literal(value, catalog)?))
}

pub(super) fn eq_placeholder(expr: &Expr) -> Option<(String, String)> {
    let (left, right) = eq_pair(expr)?;
    let (column, value) = column_and_other(left, right)?;
    Some((column, placeholder(value)?))
}

pub(super) fn binding(expr: &Expr, catalog: &SchemaCatalog) -> Option<(String, Lit)> {
    let (left, right) = eq_pair(expr)?;
    if let (Some(name), Some(value)) = (placeholder(left), literal(right, catalog)) {
        return Some((name, value));
    }
    if let (Some(name), Some(value)) = (placeholder(right), literal(left, catalog)) {
        return Some((name, value));
    }
    None
}

fn column_and_other<'a>(left: &'a Expr, right: &'a Expr) -> Option<(String, &'a Expr)> {
    match (plain_column(left), plain_column(right)) {
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

fn literals(list: &[Expr], catalog: &SchemaCatalog) -> Option<BTreeSet<Lit>> {
    list.iter().map(|expr| literal(expr, catalog)).collect()
}

fn array_literals(expr: &Expr, catalog: &SchemaCatalog) -> Option<BTreeSet<Lit>> {
    match unwrap_expr(expr) {
        Expr::Array(array) => literals(&array.elem, catalog),
        Expr::Cast {
            expr, data_type, ..
        } => {
            let values = array_literals(expr, catalog)?;
            values
                .iter()
                .all(|value| preserves(data_type, value, catalog))
                .then_some(values)
        }
        _ => None,
    }
}

pub(super) fn plain_column(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => Some(ident_key(ident)),
        _ => None,
    }
}

fn literal(expr: &Expr, catalog: &SchemaCatalog) -> Option<Lit> {
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
            let inner = literal(expr, catalog)?;
            preserves(data_type, &inner, catalog).then_some(inner)
        }
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
