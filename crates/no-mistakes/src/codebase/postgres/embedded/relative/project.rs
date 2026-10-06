use super::super::EmbeddedSqlFileFacts;
use super::package::path_inside;
use super::types::PendingRelativeCall;
use std::path::{Path, PathBuf};

/// Keep relative candidates whose resolved file is inside `package_root`.
///
/// `package_root == None` drops every candidate. Confirmed calls are left in place.
pub(crate) fn project_relative_scoped_facts(
    facts: &mut EmbeddedSqlFileFacts,
    package_root: Option<&Path>,
    mut resolve: impl FnMut(&str) -> Option<PathBuf>,
) {
    if facts.pending_relative.candidates.is_empty() {
        facts.pending_relative = Default::default();
        return;
    }
    let pending = std::mem::take(&mut facts.pending_relative);
    let Some(root) = package_root else {
        return;
    };
    let mut kept = vec![false; pending.candidates.len()];
    let mut added_name = false;
    for (index, candidate) in pending.candidates.iter().enumerate() {
        let Some(file) = resolve(&candidate.specifier) else {
            continue;
        };
        if !path_inside(&file, root) {
            continue;
        }
        kept[index] = true;
        if candidate.factory {
            facts
                .matched_factory_names
                .push(candidate.imported_name.clone());
            added_name = true;
        }
        if candidate.type_import {
            facts
                .matched_type_names
                .push(candidate.imported_name.clone());
            added_name = true;
        }
    }
    if added_name {
        sort_dedup(&mut facts.matched_factory_names);
        sort_dedup(&mut facts.matched_type_names);
    }
    let promoted = pending
        .calls
        .into_iter()
        .filter(|call| kept_owner(&kept, &call.owners))
        .collect::<Vec<_>>();
    if promoted.is_empty() {
        return;
    }
    facts.calls = merge_calls(
        std::mem::take(&mut facts.calls),
        &pending.confirmed_order,
        promoted,
    );
}

fn kept_owner(kept: &[bool], owners: &[u32]) -> bool {
    owners
        .iter()
        .any(|owner| kept.get(*owner as usize).copied().unwrap_or(false))
}

fn sort_dedup(names: &mut Vec<String>) {
    names.sort();
    names.dedup();
}

fn merge_calls(
    confirmed: Vec<super::super::EmbeddedSqlCall>,
    order: &[u32],
    promoted: Vec<PendingRelativeCall>,
) -> Vec<super::super::EmbeddedSqlCall> {
    if order.len() != confirmed.len() {
        let mut calls = confirmed;
        calls.extend(promoted.into_iter().map(|pending| pending.call));
        return calls;
    }
    let mut merged = Vec::with_capacity(confirmed.len() + promoted.len());
    let mut confirmed = confirmed.into_iter().zip(order.iter().copied()).peekable();
    let mut pending = promoted.into_iter().peekable();
    loop {
        match (confirmed.peek(), pending.peek()) {
            (Some((_, seq)), Some(call)) if call.seq < *seq => {
                merged.push(pending.next().expect("peeked pending").call);
            }
            (Some(_), _) => merged.push(confirmed.next().expect("peeked confirmed").0),
            (None, Some(_)) => merged.push(pending.next().expect("peeked pending").call),
            (None, None) => break,
        }
    }
    merged
}
