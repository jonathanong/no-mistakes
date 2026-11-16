//! Disjunctive query bounds stay per arm so their row counts add.
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{BinaryOperator, Expr};

pub(super) fn query_spans(conjuncts: &[Expr], alternatives: usize) -> Vec<Vec<Expr>> {
    let arms = alternatives.max(1);
    if !conjuncts.iter().any(contains_or) {
        return vec![conjuncts.to_vec(); arms];
    }
    let Some(branches) = and_dnf(conjuncts) else {
        return vec![shared(conjuncts); arms];
    };
    if branches.len() == arms {
        branches
    } else {
        vec![shared(conjuncts); arms]
    }
}

pub(super) fn and_conjuncts(expr: &Expr, out: &mut Vec<Expr>) {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            and_conjuncts(left, out);
            and_conjuncts(right, out);
        }
        other => out.push(other.clone()),
    }
}

pub(super) fn contains_or(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            op: BinaryOperator::Or,
            ..
        } => true,
        Expr::BinaryOp { left, right, .. } => contains_or(left) || contains_or(right),
        Expr::Between {
            expr, low, high, ..
        } => contains_or(expr) || contains_or(low) || contains_or(high),
        _ => false,
    }
}

pub(super) fn negated_between(expr: &Expr, column: &str, qualifiers: &[Vec<String>]) -> bool {
    match unwrap_expr(expr) {
        Expr::Between {
            expr,
            negated: true,
            ..
        } if super::parse::names_column(expr, column, qualifiers) => true,
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            negated_between(left, column, qualifiers) || negated_between(right, column, qualifiers)
        }
        _ => false,
    }
}

fn shared(conjuncts: &[Expr]) -> Vec<Expr> {
    conjuncts
        .iter()
        .filter(|expr| !contains_or(expr))
        .cloned()
        .collect()
}

fn and_dnf(conjuncts: &[Expr]) -> Option<Vec<Vec<Expr>>> {
    let mut acc = vec![Vec::new()];
    for conjunct in conjuncts {
        acc = combine(acc, &dnf(conjunct)?)?;
    }
    Some(acc)
}

fn dnf(expr: &Expr) -> Option<Vec<Vec<Expr>>> {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => combine(dnf(left)?, &dnf(right)?),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Or,
            right,
        } => {
            let mut left = dnf(left)?;
            let right = dnf(right)?;
            if left.len() + right.len() > 32 {
                return None;
            }
            left.extend(right);
            Some(left)
        }
        other => Some(vec![vec![other.clone()]]),
    }
}

fn combine(left: Vec<Vec<Expr>>, right: &[Vec<Expr>]) -> Option<Vec<Vec<Expr>>> {
    if left.len().saturating_mul(right.len()) > 32 {
        return None;
    }
    let mut out = Vec::with_capacity(left.len().saturating_mul(right.len()));
    for left in left {
        for right in right {
            let mut branch = left.clone();
            branch.extend(right.iter().cloned());
            out.push(branch);
        }
    }
    Some(out)
}
