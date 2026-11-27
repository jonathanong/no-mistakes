//! Resolve each ON before later JOIN children can change its lexical namespace.
use super::{Frame, Qualified, SqlBareRead};
use sqlparser::ast::{Expr, JoinConstraint, JoinOperator, TableWithJoins};
use std::collections::BTreeMap;

pub(super) struct OnReads {
    qualifiers: usize,
    reads: usize,
    bare: BTreeMap<String, usize>,
}

pub(super) fn register(frame: &mut Frame, from: &TableWithJoins) {
    for join in &from.joins {
        let condition = match &join.join_operator {
            JoinOperator::Join(JoinConstraint::On(expr))
            | JoinOperator::Inner(JoinConstraint::On(expr))
            | JoinOperator::Left(JoinConstraint::On(expr))
            | JoinOperator::LeftOuter(JoinConstraint::On(expr))
            | JoinOperator::Right(JoinConstraint::On(expr))
            | JoinOperator::RightOuter(JoinConstraint::On(expr))
            | JoinOperator::FullOuter(JoinConstraint::On(expr)) => expr,
            _ => continue,
        };
        // This address belongs to the borrowed AST and is used only in this visitor run.
        frame
            .on_conditions
            .insert(condition as *const Expr as usize, None);
    }
}

pub(super) fn begin(frame: &mut Frame, expr: &Expr) {
    let key = expr as *const Expr as usize;
    if frame.on_conditions.contains_key(&key) {
        let reads = OnReads {
            qualifiers: frame.qualifiers.len(),
            reads: frame.reads.len(),
            // Keep the outer expression counts without cloning them once per JOIN.
            bare: std::mem::take(&mut frame.bare),
        };
        frame.on_conditions.insert(key, Some(reads));
    }
}

pub(super) fn finish(frame: &mut Frame, expr: &Expr) {
    let Some(Some(before)) = frame.on_conditions.remove(&(expr as *const Expr as usize)) else {
        return;
    };
    let candidates = frame.scope.qualified_candidates();
    frame.join_qualifiers.extend(
        frame
            .qualifiers
            .split_off(before.qualifiers)
            .into_iter()
            .filter(|read| !frame.scope.relations.contains(&read.key))
            .map(|mut read: Qualified| {
                read.scopes.push(candidates.clone());
                read
            }),
    );
    let mut reads = frame.reads.split_off(before.reads);
    for (column, _) in std::mem::replace(&mut frame.bare, before.bare) {
        if !frame.scope.whole_rows.contains(&column) {
            reads.push(SqlBareRead {
                column,
                tables: Vec::new(),
            });
        }
    }
    frame.scope.resolve_reads(&mut reads);
    frame.join_reads.extend(reads);
}
