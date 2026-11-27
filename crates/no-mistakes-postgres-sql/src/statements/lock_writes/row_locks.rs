use super::{
    helpers::{access, generated_name, qualifier},
    scope,
    sequences::Pending,
    RowAccess, SequenceMode,
};
use crate::idents::object_name_identity;
use crate::lock_inventory::tokens::RowLockMode;
use crate::statements::value::PlaceholderPositions;
use sqlparser::ast::{LockType, Query, SetExpr};
use std::collections::BTreeSet;

mod explicit;
use explicit::collect_explicit;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;

pub(super) fn collect_advanced(
    query: &Query,
    positions: PlaceholderPositions<'_>,
    scopes: &[BTreeSet<String>],
    locks: &mut Vec<RowAccess>,
    mut sequence: Option<&mut Pending<'_, '_>>,
    top_level: bool,
) {
    let modes = sequence.as_ref().and_then(|sequence| sequence.modes(query));
    let track = sequence.is_some();
    let rows = if query.locks.iter().all(|lock| lock.of.is_some()) {
        collect_explicit(query, positions, modes.as_deref(), track)
    } else {
        let row = collect_legacy(query, positions, true);
        let mode = if row.relation.is_some() {
            strongest_mode(query, modes.as_deref(), track)
        } else {
            unknown_share(query, track)
        };
        vec![(row, mode)]
    };
    for (mut lock, mode) in rows {
        scope::hide_cte(&mut lock, scopes);
        if let (Some(sequence), Some(mode)) = (sequence.as_deref_mut(), mode) {
            if mode != SequenceMode::Update || (top_level && guaranteed_update(query, &lock)) {
                sequence.lock(mode, top_level.then_some(locks.len()));
            }
        }
        locks.push(lock);
    }
}

fn strongest_mode(
    query: &Query,
    modes: Option<&[RowLockMode]>,
    track: bool,
) -> Option<SequenceMode> {
    if !track {
        return None;
    }
    let Some(modes) = modes else {
        return unknown_share(query, true);
    };
    modes.iter().copied().max().and_then(sequence_mode)
}

fn sequence_mode(mode: RowLockMode) -> Option<SequenceMode> {
    Some(match mode {
        RowLockMode::KeyShare => SequenceMode::KeyShare,
        RowLockMode::Share => SequenceMode::Share,
        RowLockMode::Update => SequenceMode::Update,
        RowLockMode::NoKeyUpdate => return None,
    })
}

fn guaranteed_update(query: &Query, row: &RowAccess) -> bool {
    let SetExpr::Select(select) = query.body.as_ref() else {
        return false;
    };
    let [table] = select.from.as_slice() else {
        return false;
    };
    table.joins.is_empty()
        && row.key.is_some()
        && query.with.is_none()
        && query.limit_clause.is_none()
        && query.fetch.is_none()
        && query.locks.iter().all(|lock| lock.nonblock.is_none())
}

pub(super) fn collect_legacy(
    query: &Query,
    positions: PlaceholderPositions<'_>,
    allow: bool,
) -> RowAccess {
    if !allow {
        return unresolved();
    }
    let SetExpr::Select(select) = query.body.as_ref() else {
        return unresolved();
    };
    match select.from.as_slice() {
        [from] if from.joins.is_empty() => {
            let row = access(&from.relation, select.selection.as_ref(), positions);
            let valid = query.locks.iter().all(|lock| {
                lock.of.as_ref().is_none_or(|target| {
                    !generated_name(target, positions)
                        && qualifier(&from.relation)
                            .is_some_and(|name| name == object_name_identity(target))
                })
            });
            if valid {
                row
            } else {
                unresolved()
            }
        }
        _ => unresolved(),
    }
}

fn unknown_share(query: &Query, track: bool) -> Option<SequenceMode> {
    (track
        && query
            .locks
            .iter()
            .any(|lock| lock.lock_type == LockType::Share))
    .then_some(SequenceMode::UnknownShare)
}

fn unresolved() -> RowAccess {
    RowAccess {
        relation: None,
        key: None,
        recovered_key: None,
        key_identity_unproven: false,
        has_predicate: false,
        only: false,
    }
}
