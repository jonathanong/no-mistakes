use super::{plain_table, qualified_column};
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::SchemaCatalog;
use sqlparser::ast::{BinaryOperator, Expr, Join, JoinConstraint, JoinOperator, Value};

pub(super) fn preserves_rows(join: &Join, base_quals: &[String], catalog: &SchemaCatalog) -> bool {
    if join.global {
        return false;
    }
    let constraint = match &join.join_operator {
        JoinOperator::Left(constraint) | JoinOperator::LeftOuter(constraint) => constraint,
        _ => return false,
    };
    let JoinConstraint::On(expr) = constraint else {
        return false;
    };
    let Some((name, quals)) = plain_table(&join.relation) else {
        return false;
    };
    if catalog.view_relation(&name).is_some() {
        return false;
    }
    let Some(primary_key) = catalog
        .bounded_relation(&name)
        .and_then(|table| table.primary_key.clone())
        .filter(|key| !key.is_empty())
    else {
        return false;
    };
    let mut matched = Vec::new();
    if !equalities(expr, &quals, base_quals, &primary_key, &mut matched) {
        return false;
    }
    primary_key
        .iter()
        .all(|column| matched.iter().any(|got| got.eq_ignore_ascii_case(column)))
}

fn equalities(
    expr: &Expr,
    join_quals: &[String],
    base_quals: &[String],
    primary_key: &[String],
    matched: &mut Vec<String>,
) -> bool {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            equalities(left, join_quals, base_quals, primary_key, matched)
                && equalities(right, join_quals, base_quals, primary_key, matched)
        }
        Expr::BinaryOp {
            op: BinaryOperator::Or,
            ..
        } => false,
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } => {
            if let Some(column) = pk_equality(left, right, join_quals, base_quals, primary_key) {
                matched.push(column);
            }
            true
        }
        _ => true,
    }
}

fn pk_equality(
    left: &Expr,
    right: &Expr,
    join_quals: &[String],
    base_quals: &[String],
    primary_key: &[String],
) -> Option<String> {
    for (join_side, other) in [(left, right), (right, left)] {
        let Some(column) = qualified_column(join_side, join_quals) else {
            continue;
        };
        if primary_key
            .iter()
            .any(|key| key.eq_ignore_ascii_case(&column))
            && (qualified_column(other, base_quals).is_some() || literal(other))
        {
            return Some(column);
        }
    }
    None
}

fn literal(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => !matches!(value.value, Value::Null),
        _ => false,
    }
}
