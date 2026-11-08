use super::super::super::value::PlaceholderPositions;
use super::super::Scope;
use super::{extract_conjunct, Resolver};
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::SqlBoundPin;
use sqlparser::ast::{BinaryOperator, Expr, UnaryOperator, Value};
use std::collections::BTreeMap;

type Ranges<'a> = BTreeMap<(usize, String, String), (u8, &'a Expr, &'a Expr)>;

pub(in super::super) fn extract(
    expr: &Expr,
    resolver: &Resolver,
    restricted: &[usize],
    scope: &Scope,
    positions: PlaceholderPositions<'_>,
    out: &mut Vec<(usize, SqlBoundPin)>,
) {
    extract_all(&[expr], resolver, restricted, scope, positions, out);
}

pub(in super::super) fn extract_all(
    conjuncts: &[&Expr],
    resolver: &Resolver,
    restricted: &[usize],
    scope: &Scope,
    positions: PlaceholderPositions<'_>,
    out: &mut Vec<(usize, SqlBoundPin)>,
) {
    let mut ranges = BTreeMap::new();
    for expr in conjuncts {
        extract_conjunct(expr, resolver, restricted, scope, positions, out);
        collect(expr, resolver, &mut ranges);
    }
    for (_, (mask, column, value)) in ranges {
        if mask == 3 {
            let equality = Expr::BinaryOp {
                left: Box::new(column.clone()),
                op: BinaryOperator::Eq,
                right: Box::new(value.clone()),
            };
            extract_conjunct(&equality, resolver, restricted, scope, positions, out);
        }
    }
}

fn collect<'a>(expr: &'a Expr, resolver: &Resolver, ranges: &mut Ranges<'a>) {
    let Expr::BinaryOp { left, op, right } = unwrap_expr(expr) else {
        return;
    };
    if *op == BinaryOperator::And {
        collect(left, resolver, ranges);
        collect(right, resolver, ranges);
        return;
    }
    let mask = match op {
        BinaryOperator::GtEq => 1,
        BinaryOperator::LtEq => 2,
        _ => return,
    };
    for (column, value, mask) in [(left, right, mask), (right, left, 3 ^ mask)] {
        let (Some((item, name)), Some(value_key)) = (resolver.column(column), scalar_key(value))
        else {
            continue;
        };
        let entry = ranges
            .entry((item, name, value_key))
            .or_insert((0, column, value));
        entry.0 |= mask;
    }
}

fn scalar_key(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Value(value) => match &value.value {
            Value::Placeholder(name)
                if !name.starts_with('$')
                    || name.len() == 1
                    || !name[1..].bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                None
            }
            value => Some(value.to_string()),
        },
        Expr::UnaryOp {
            op: UnaryOperator::Plus | UnaryOperator::Minus,
            expr: inner,
        } if matches!(unwrap_expr(inner), Expr::Value(value) if matches!(value.value, Value::Number(..))) => {
            Some(expr.to_string())
        }
        _ => None,
    }
}
