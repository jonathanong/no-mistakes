use super::{parse_postgres_expression, snapshot::SnapshotIndex};
use crate::codebase::postgres::parse_postgres_sql;
use sqlparser::ast::{visit_expressions, visit_expressions_mut};
use sqlparser::ast::{BinaryOperator, Expr, ObjectNamePart, Statement, Value};
use std::ops::ControlFlow;

/// Prepared catalog key evidence shared by bounded proofs and schema policies.
#[derive(Debug, Clone)]
pub(crate) struct BoundedUniqueIndex {
    pub(crate) name: String,
    pub(crate) keys: Vec<Expr>,
    pub(crate) predicate: Vec<Expr>,
    pub(crate) nulls_not_distinct: bool,
}

pub(super) fn from_snapshot(name: &str, index: &SnapshotIndex) -> Option<BoundedUniqueIndex> {
    let supported = (index.unique || index.primary)
        && index.valid
        && index.ready
        && index.live.unwrap_or(true)
        && index.immediate.unwrap_or(true)
        && index.access_method.eq_ignore_ascii_case("btree")
        && index
            .keys
            .iter()
            .all(|key| key.ordering_supported.unwrap_or(true));
    if !supported || index.keys.is_empty() {
        return None;
    }
    let keys = index
        .keys
        .iter()
        .map(|key| {
            let expr = parse_postgres_expression(&key.expression)?;
            (!contains_placeholder(&expr)).then(|| canonical_bound_expr(&expr))
        })
        .collect::<Option<Vec<_>>>()?;
    let predicate = match index.predicate.as_deref() {
        Some(text) => {
            let expr = parse_postgres_expression(text)?;
            if contains_placeholder(&expr) {
                return None;
            }
            Some(canonical_bound_expr(&expr))
        }
        None => None,
    };
    let mut conjuncts = Vec::new();
    if let Some(predicate) = predicate {
        conjuncts_of(&predicate, &mut conjuncts);
    }
    let nulls_not_distinct = parse_postgres_sql(&index.definition)
        .ok()
        .and_then(|statements| statements.into_iter().next())
        .is_some_and(|statement| {
            matches!(statement, Statement::CreateIndex(index) if index.nulls_distinct == Some(false))
        });
    // The existing plain-key path already covers these indexes without extra AST evidence.
    if conjuncts.is_empty()
        && !nulls_not_distinct
        && index
            .keys
            .iter()
            .all(|key| super::names::plain_column(&key.expression).is_some())
    {
        return None;
    }
    Some(BoundedUniqueIndex {
        name: name.to_string(),
        keys,
        predicate: conjuncts,
        nulls_not_distinct,
    })
}

fn contains_placeholder(expr: &Expr) -> bool {
    visit_expressions(expr, |child| {
        if matches!(child, Expr::Value(value) if matches!(value.value, Value::Placeholder(_))) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    })
    .is_break()
}

pub(crate) fn conjuncts_of(expr: &Expr, out: &mut Vec<Expr>) {
    match expr {
        Expr::Nested(inner) => conjuncts_of(inner, out),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            conjuncts_of(left, out);
            conjuncts_of(right, out);
        }
        other => out.push(unnest(other)),
    }
}

pub(crate) fn unnest(expr: &Expr) -> Expr {
    match expr {
        Expr::Nested(inner) => unnest(inner),
        other => other.clone(),
    }
}

pub(crate) fn canonical_bound_expr(expr: &Expr) -> Expr {
    let mut normalized = unnest(expr);
    let _: ControlFlow<()> = visit_expressions_mut(&mut normalized, |expr| {
        match expr {
            Expr::Identifier(ident) if ident.quote_style.is_none() => {
                ident.value.make_ascii_lowercase();
            }
            Expr::Function(function) => {
                for part in &mut function.name.0 {
                    match part {
                        ObjectNamePart::Identifier(ident) if ident.quote_style.is_none() => {
                            ident.value.make_ascii_lowercase();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        ControlFlow::Continue(())
    });
    normalized
}
