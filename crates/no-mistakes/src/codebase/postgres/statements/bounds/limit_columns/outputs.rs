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
    name_columns(&prove(query, positions, parent)?, alias_columns)
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
    if select.projection.iter().any(|item| {
        matches!(
            item,
            SelectItem::Wildcard(_) | SelectItem::QualifiedWildcard(_, _)
        )
    }) {
        return None;
    }
    let mut visible = parent.clone();
    visible.absorb(&select.from, positions);
    let mut proved = Vec::new();
    for item in &select.projection {
        let (expr, name) = match item {
            SelectItem::UnnamedExpr(expr) => (expr, expr_name(expr)),
            SelectItem::ExprWithAlias { expr, alias } => (expr, Some(ident_key(alias))),
            _ => return None,
        };
        proved.push((name, project_kind(expr, positions, &visible)));
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
            .map(|expr| project_kind(expr, positions, parent))
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
) -> Option<FixedCount> {
    fixed_count_at(expr, positions, &|column| {
        if let Some(kind) = visible.contains(column) {
            Some(kind)
        } else if matches!(column, Expr::Identifier(_) | Expr::CompoundIdentifier(_)) {
            Some(FixedCount::Nullable)
        } else {
            None
        }
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
                let name = match (left_name, right_name) {
                    (Some(left), Some(right)) if left == right => Some(left),
                    (Some(name), None) | (None, Some(name)) => Some(name),
                    _ => None,
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

fn name_columns(
    proved: &Proof,
    alias_columns: &[String],
) -> Option<BTreeMap<String, Option<FixedCount>>> {
    let names = if alias_columns.is_empty() {
        proved
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Option<Vec<_>>>()?
    } else if alias_columns.len() == proved.len() {
        alias_columns.to_vec()
    } else {
        return None;
    };
    if names.len()
        != names
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    {
        return None;
    }
    Some(
        names
            .into_iter()
            .zip(proved.iter().map(|(_, kind)| *kind))
            .collect(),
    )
}

fn expr_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Nested(inner) | Expr::Cast { expr: inner, .. } => expr_name(inner),
        Expr::Identifier(ident) => Some(ident_key(ident)),
        Expr::CompoundIdentifier(parts) => parts.last().map(ident_key),
        _ => None,
    }
}
