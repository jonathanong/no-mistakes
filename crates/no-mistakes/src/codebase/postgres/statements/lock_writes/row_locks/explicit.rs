use super::{
    access, generated_name, object_name_identity, sequence_mode, unknown_share, unresolved,
    PlaceholderPositions, RowAccess, RowLockMode, SequenceMode,
};
use crate::codebase::postgres::idents::{ident_key, relation_part_key};
use sqlparser::ast::{
    BinaryOperator, Expr, LockClause, LockType, ObjectNamePart, Query, Select, SetExpr, TableFactor,
};
use std::collections::BTreeMap;

pub(super) fn collect_explicit(
    query: &Query,
    positions: PlaceholderPositions<'_>,
    modes: Option<&[RowLockMode]>,
    track_share: bool,
) -> Vec<(RowAccess, Option<SequenceMode>)> {
    let SetExpr::Select(select) = query.body.as_ref() else {
        return vec![(unresolved(), unknown_share(query, track_share))];
    };
    let relations = relation_index(select);
    let target = |lock: &LockClause| {
        let name = lock
            .of
            .as_ref()
            .filter(|target| !generated_name(target, positions))?;
        let [factor] = relations.get(&object_name_identity(name))?.as_slice() else {
            return None;
        };
        Some(*factor)
    };
    // Each factor represents a distinct FROM target, even when two resolve to the same relation.
    let upgrades: crate::fx::FxHashSet<_> = query
        .locks
        .iter()
        .filter(|lock| track_share && lock.lock_type == LockType::Update)
        .filter_map(|lock| target(lock).map(std::ptr::from_ref))
        .collect();
    let mut locks: Vec<(RowAccess, Option<SequenceMode>)> = Vec::new();
    for (index, lock) in query.locks.iter().enumerate() {
        let Some(factor) = target(lock) else {
            return vec![(unresolved(), unknown_share(query, track_share))];
        };
        let mut row = access(factor, select.selection.as_ref(), positions);
        if query.locks.len() > 1 && row.key.is_none() {
            if let Some(keyed) = select
                .selection
                .as_ref()
                .and_then(|predicate| conjunctive_key(factor, predicate, positions))
            {
                row.key = keyed.key;
                row.recovered_key = keyed.recovered_key;
                row.key_identity_unproven |= keyed.key_identity_unproven;
            }
        }
        let mode = if let Some(modes) = modes {
            match modes.get(index).copied().and_then(sequence_mode) {
                Some(SequenceMode::Share | SequenceMode::KeyShare)
                    if upgrades.contains(&std::ptr::from_ref(factor)) =>
                {
                    None
                }
                mode => mode,
            }
        } else if track_share
            && lock.lock_type == LockType::Share
            && !upgrades.contains(&std::ptr::from_ref(factor))
        {
            Some(SequenceMode::UnknownShare)
        } else {
            None
        };
        if let Some((_, prior)) = locks.iter_mut().find(|(candidate, _)| *candidate == row) {
            if prior.is_none()
                || *prior == Some(SequenceMode::Update)
                || mode == Some(SequenceMode::Share)
            {
                *prior = mode.or(*prior);
            }
        } else {
            locks.push((row, mode));
        }
    }
    locks
}

fn conjunctive_key(
    factor: &TableFactor,
    predicate: &Expr,
    positions: PlaceholderPositions<'_>,
) -> Option<RowAccess> {
    match super::super::unwrap_expr(predicate) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => conjunctive_key(factor, left, positions)
            .or_else(|| conjunctive_key(factor, right, positions)),
        expression => {
            let row = access(factor, Some(expression), positions);
            row.key.is_some().then_some(row)
        }
    }
}

fn relation_index(select: &Select) -> BTreeMap<String, Vec<&TableFactor>> {
    let mut index = BTreeMap::<String, Vec<&TableFactor>>::new();
    for from in &select.from {
        for factor in
            std::iter::once(&from.relation).chain(from.joins.iter().map(|join| &join.relation))
        {
            let TableFactor::Table {
                name,
                alias,
                args: None,
                ..
            } = factor
            else {
                continue;
            };
            if let Some(alias) = alias {
                index
                    .entry(normalized_ident(&alias.name))
                    .or_default()
                    .push(factor);
                continue;
            }
            let table_key = object_name_identity(name);
            index.entry(table_key.clone()).or_default().push(factor);
            if let Some(leaf) = name.0.last().and_then(ObjectNamePart::as_ident) {
                let leaf_key = normalized_ident(leaf);
                if leaf_key != table_key {
                    index.entry(leaf_key).or_default().push(factor);
                }
            }
        }
    }
    index
}

fn normalized_ident(ident: &sqlparser::ast::Ident) -> String {
    relation_part_key(&ident_key(ident))
}
