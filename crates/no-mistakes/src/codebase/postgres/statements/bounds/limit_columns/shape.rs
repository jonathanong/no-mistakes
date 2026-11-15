use crate::codebase::postgres::idents::ident_key;
use crate::codebase::postgres::statements::limit::FixedCount;
use sqlparser::ast::{Expr, GroupByExpr, GroupByWithModifier, Select};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn grouping_adds_nulls(select: &Select) -> bool {
    match &select.group_by {
        GroupByExpr::Expressions(exprs, modifiers) => {
            exprs.iter().any(expr_adds_nulls) || modifiers.iter().any(modifier_adds_nulls)
        }
        GroupByExpr::All(modifiers) => modifiers.iter().any(modifier_adds_nulls),
    }
}

fn modifier_adds_nulls(modifier: &GroupByWithModifier) -> bool {
    matches!(
        modifier,
        GroupByWithModifier::Rollup
            | GroupByWithModifier::Cube
            | GroupByWithModifier::GroupingSets(_)
    )
}

fn expr_adds_nulls(expr: &Expr) -> bool {
    match expr {
        Expr::Rollup(_) | Expr::Cube(_) | Expr::GroupingSets(_) => true,
        Expr::Nested(inner) => expr_adds_nulls(inner),
        _ => false,
    }
}

pub(super) fn name_columns(
    proved: &[(Option<String>, Option<FixedCount>)],
    alias_columns: &[String],
) -> Option<BTreeMap<String, Option<FixedCount>>> {
    if alias_columns.len() > proved.len() {
        return None;
    }
    let mut pairs = Vec::new();
    for (index, (name, kind)) in proved.iter().enumerate() {
        let column = if let Some(alias) = alias_columns.get(index) {
            alias.clone()
        } else if alias_columns.is_empty() {
            name.clone()?
        } else {
            match name.clone() {
                Some(name) => name,
                None => break,
            }
        };
        pairs.push((column, *kind));
    }
    if pairs.len()
        != pairs
            .iter()
            .map(|(name, _)| name)
            .collect::<BTreeSet<_>>()
            .len()
    {
        return None;
    }
    Some(pairs.into_iter().collect())
}

pub(super) fn expr_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Nested(inner) | Expr::Cast { expr: inner, .. } => expr_name(inner),
        Expr::Identifier(ident) => Some(ident_key(ident)),
        Expr::CompoundIdentifier(parts) => parts.last().map(ident_key),
        _ => None,
    }
}
