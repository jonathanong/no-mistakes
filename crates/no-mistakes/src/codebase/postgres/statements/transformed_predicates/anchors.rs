use super::super::row_variant::is_row_variant;
use super::super::select::uses::{columns::bare, scope::BaseRel};
use super::super::value::PlaceholderPositions;
use super::super::SqlTransformedColumnProbeFact as Probe;
use super::scope::{base_relations, owner, push_factor, qualifier};
use sqlparser::ast::{visit_expressions, BinaryOperator as Op, Expr, JoinOperator, TableWithJoins};
use std::ops::ControlFlow;

pub(super) struct Anchor {
    occurrence: String,
    probe: Probe,
}

pub(super) fn collect(
    tables: &[TableWithJoins],
    selection: Option<&Expr>,
    ctes: &[String],
    positions: PlaceholderPositions<'_>,
) -> Vec<Anchor> {
    let mut out = Vec::new();
    if let Some(expr) = selection {
        conjuncts(expr, &base_relations(tables, ctes), positions, &mut out);
    }
    for table in tables {
        let mut visible = Vec::new();
        push_factor(&table.relation, ctes, &mut visible);
        for join in &table.joins {
            let start = visible.len();
            push_factor(&join.relation, ctes, &mut visible);
            if !matches!(
                join.join_operator,
                JoinOperator::Join(_)
                    | JoinOperator::Inner(_)
                    | JoinOperator::Left(_)
                    | JoinOperator::LeftOuter(_)
            ) {
                continue;
            }
            if let Some(expr) = super::super::select::join_expr(&join.join_operator) {
                let mut joined = Vec::new();
                conjuncts(expr, &visible, positions, &mut joined);
                out.extend(joined.into_iter().filter(|anchor| {
                    visible[start..]
                        .iter()
                        .any(|rel| key(rel) == anchor.occurrence)
                }));
            }
        }
    }
    out
}

fn conjuncts(
    expr: &Expr,
    rels: &[BaseRel],
    positions: PlaceholderPositions<'_>,
    out: &mut Vec<Anchor>,
) {
    match expr {
        Expr::Nested(inner) => conjuncts(inner, rels, positions, out),
        Expr::BinaryOp {
            left,
            op: Op::And,
            right,
        } => {
            conjuncts(left, rels, positions, out);
            conjuncts(right, rels, positions, out);
        }
        Expr::BinaryOp {
            left,
            op: Op::Eq | Op::Gt | Op::GtEq | Op::Lt | Op::LtEq,
            right,
        } => {
            record(left, right, rels, positions, out);
            record(right, left, rels, positions, out);
        }
        Expr::AnyOp {
            left,
            compare_op: Op::Eq,
            right,
            ..
        } => {
            record(left, right, rels, positions, out);
        }
        Expr::Between {
            expr,
            negated: false,
            low,
            high,
        } if independent(low, occurrence(rels, expr), rels, positions) => {
            record(expr, high, rels, positions, out);
        }
        _ => {}
    }
}

fn record(
    expr: &Expr,
    other: &Expr,
    rels: &[BaseRel],
    positions: PlaceholderPositions<'_>,
    out: &mut Vec<Anchor>,
) {
    let Some((_, column)) = bare(expr, positions) else {
        return;
    };
    let Some(occurrence) = occurrence(rels, expr) else {
        return;
    };
    if !independent(other, Some(occurrence), rels, positions) {
        return;
    }
    let (table, candidate_tables) = owner(rels, qualifier(expr).as_deref());
    out.push(Anchor {
        occurrence: occurrence.to_string(),
        probe: Probe {
            table,
            column,
            candidate_tables,
        },
    });
}

fn independent(
    expr: &Expr,
    target: Option<&str>,
    rels: &[BaseRel],
    positions: PlaceholderPositions<'_>,
) -> bool {
    visit_expressions(expr, |child| {
        let unsafe_value = match child {
            Expr::Subquery(_)
            | Expr::Exists { .. }
            | Expr::InSubquery { .. }
            | Expr::Collate { .. } => true,
            Expr::Function(function) if is_row_variant(&function.name) => true,
            Expr::Identifier(_) | Expr::CompoundIdentifier(_)
                if bare(child, positions).is_some() =>
            {
                occurrence(rels, child).is_none_or(|source| Some(source) == target)
            }
            _ => false,
        };
        if unsafe_value {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    })
    .is_continue()
}

fn key(rel: &BaseRel) -> &str {
    rel.alias.as_deref().unwrap_or(&rel.table)
}

fn occurrence<'a>(rels: &'a [BaseRel], expr: &Expr) -> Option<&'a str> {
    let qualifier = qualifier(expr);
    let mut matches = rels.iter().filter(|rel| {
        qualifier.as_deref().map_or(rels.len() == 1, |qualifier| {
            let (table, _) = owner(std::slice::from_ref(*rel), Some(qualifier));
            !table.is_empty()
        })
    });
    match (matches.next(), matches.next()) {
        (Some(rel), None) if !rel.unknown => Some(key(rel)),
        _ => None,
    }
}

pub(super) fn for_column(anchors: &[Anchor], rels: &[BaseRel], expr: &Expr) -> Vec<Probe> {
    let Some(occurrence) = occurrence(rels, expr) else {
        return Vec::new();
    };
    anchors
        .iter()
        .filter(|anchor| anchor.occurrence == occurrence)
        .map(|anchor| anchor.probe.clone())
        .collect()
}
