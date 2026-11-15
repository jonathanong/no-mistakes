use super::Relations;
use crate::codebase::postgres::idents::ident_key;
use crate::codebase::postgres::statements::limit::{fixed_count_at, FixedCount};
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::{Expr, Query, Select, SelectItem, SetExpr};
use std::collections::BTreeMap;

type Proof = Vec<(Option<String>, Option<FixedCount>)>;

pub(in crate::codebase::postgres::statements::bounds) fn of_query(
    query: &Query,
    positions: PlaceholderPositions<'_>,
    parent: &Relations,
    alias_columns: &[String],
) -> Option<BTreeMap<String, Option<FixedCount>>> {
    super::shape::name_columns(&prove(query, positions, parent)?, alias_columns)
}

fn prove(query: &Query, positions: PlaceholderPositions<'_>, parent: &Relations) -> Option<Proof> {
    let mut visible = parent.clone();
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            let name = ident_key(&cte.alias.name);
            let columns = cte
                .alias
                .columns
                .iter()
                .map(|column| ident_key(&column.name))
                .collect::<Vec<_>>();
            let proof = if with.recursive {
                None
            } else {
                of_query(&cte.query, positions, &visible, &columns)
            };
            visible.record(name, proof);
        }
    }
    prove_set(&query.body, positions, &visible)
}

fn prove_set(
    set: &SetExpr,
    positions: PlaceholderPositions<'_>,
    parent: &Relations,
) -> Option<Proof> {
    match set {
        SetExpr::Select(select) => prove_select(select, positions, parent),
        SetExpr::Values(values) => prove_values(values, positions, parent),
        SetExpr::SetOperation { left, right, .. } => merge_proofs(
            prove_set(left, positions, parent)?,
            prove_set(right, positions, parent)?,
        ),
        SetExpr::Query(query) => prove(query, positions, parent),
        _ => None,
    }
}

fn prove_select(
    select: &Select,
    positions: PlaceholderPositions<'_>,
    parent: &Relations,
) -> Option<Proof> {
    let mut visible = parent.clone();
    visible.absorb(&select.from, positions);
    let drop_columns = super::shape::grouping_adds_nulls(select);
    let mut proved = Vec::new();
    for item in &select.projection {
        let (expr, name) = match item {
            SelectItem::UnnamedExpr(expr) => (expr, super::shape::expr_name(expr)),
            SelectItem::ExprWithAlias { expr, alias } => (expr, Some(ident_key(alias))),
            _ => return None,
        };
        proved.push((name, project_kind(expr, positions, &visible, drop_columns)));
    }
    Some(proved)
}

fn prove_values(
    values: &sqlparser::ast::Values,
    positions: PlaceholderPositions<'_>,
    parent: &Relations,
) -> Option<Proof> {
    let mut proved: Option<Vec<Option<FixedCount>>> = None;
    for row in &values.rows {
        let kinds = row
            .iter()
            .map(|expr| project_kind(expr, positions, parent, false))
            .collect::<Vec<_>>();
        proved = Some(match proved {
            None => kinds,
            Some(previous) if previous.len() == kinds.len() => previous
                .into_iter()
                .zip(kinds)
                .map(|(left, right)| merge_kind(left, right))
                .collect(),
            Some(_) => return None,
        });
    }
    proved.map(|kinds| kinds.into_iter().map(|kind| (None, kind)).collect())
}

fn project_kind(
    expr: &Expr,
    positions: PlaceholderPositions<'_>,
    visible: &Relations,
    drop_columns: bool,
) -> Option<FixedCount> {
    fixed_count_at(expr, positions, &|column| {
        if drop_columns {
            return Some(FixedCount::Nullable);
        }
        // Only identifiers reach this callback. An unknown one is nullable, not a proof.
        Some(visible.contains(column).unwrap_or(FixedCount::Nullable))
    })
    .filter(|kind| matches!(kind, FixedCount::Caller | FixedCount::NonNull))
}

fn merge_proofs(left: Proof, right: Proof) -> Option<Proof> {
    if left.len() != right.len() {
        return None;
    }
    Some(
        left.into_iter()
            .zip(right)
            .map(|((left_name, left_kind), (right_name, right_kind))| {
                // PostgreSQL names a set-operation output from the left arm.
                let name = if left_name.is_some() {
                    left_name
                } else {
                    right_name
                };
                (name, merge_kind(left_kind, right_kind))
            })
            .collect(),
    )
}

fn merge_kind(left: Option<FixedCount>, right: Option<FixedCount>) -> Option<FixedCount> {
    match (left, right) {
        (Some(FixedCount::NonNull), Some(FixedCount::NonNull)) => Some(FixedCount::NonNull),
        (
            Some(FixedCount::NonNull | FixedCount::Caller),
            Some(FixedCount::NonNull | FixedCount::Caller),
        ) => Some(FixedCount::Caller),
        _ => None,
    }
}
