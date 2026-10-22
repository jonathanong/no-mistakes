use super::super::EmbeddedSqlFileFacts;
use super::package::path_inside;
use super::types::PendingRelativeCall;
use std::path::{Path, PathBuf};
mod diagnostic;

/// Keep relative candidates whose resolved file is inside `package_root`.
///
/// `package_root == None` drops every candidate. Confirmed calls are left in place.
pub(crate) fn project_relative_scoped_facts(
    facts: &mut EmbeddedSqlFileFacts,
    source: &str,
    package_root: Option<&Path>,
    mut resolve: impl FnMut(&str) -> Option<PathBuf>,
) {
    if facts.pending_relative.candidates.is_empty() {
        facts.pending_relative = Default::default();
        return;
    }
    let pending = std::mem::take(&mut facts.pending_relative);
    if package_root.is_none() {
        facts
            .undeclared_executor_types
            .retain(|entry| entry.owner.is_none());
    }
    let Some(root) = package_root else {
        add_unproven(facts, source, diagnostic::unproven_spans(&pending, &[]));
        return;
    };
    let mut diagnostic_owners: crate::fx::FxHashSet<u32> = facts
        .undeclared_executor_types
        .iter()
        .filter_map(|entry| entry.owner)
        .collect();
    diagnostic_owners.extend(
        pending
            .calls
            .iter()
            .flat_map(|call| call.owners.iter().copied()),
    );
    let mut kept = vec![false; pending.candidates.len()];
    let mut added_name = false;
    for (index, candidate) in pending.candidates.iter().enumerate() {
        if !candidate.factory
            && !candidate.type_import
            && !diagnostic_owners.contains(&(index as u32))
        {
            continue;
        }
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
    facts.undeclared_executor_types.retain_mut(|entry| {
        let Some(owner) = entry.owner.take() else {
            return true;
        };
        let proven = kept.get(owner as usize).copied().unwrap_or(false);
        proven
    });
    if added_name {
        sort_dedup(&mut facts.matched_factory_names);
        sort_dedup(&mut facts.matched_type_names);
    }
    add_unproven(facts, source, diagnostic::unproven_spans(&pending, &kept));
    let promoted = pending
        .calls
        .into_iter()
        .filter(|call| kept_owner(&kept, &call.owners))
        .map(|call| {
            let start = pending
                .call_spans
                .get(&call.seq)
                .copied()
                .unwrap_or_default();
            (call, start)
        })
        .collect::<Vec<_>>();
    if promoted.is_empty() {
        return;
    }
    (facts.calls, facts.call_spans) = merge_calls(
        std::mem::take(&mut facts.calls)
            .into_iter()
            .zip(std::mem::take(&mut facts.call_spans))
            .collect(),
        &pending.confirmed_order,
        promoted,
    );
}

fn add_unproven(facts: &mut EmbeddedSqlFileFacts, source: &str, spans: Vec<(u32, u32)>) {
    if !facts.record_unproven_spans {
        return;
    }
    for span in spans {
        if super::super::unproven::keep_unproven_span(source, span, &facts.postgres_rule_ids) {
            facts.unproven_member_query_spans.push(span);
        }
    }
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
    confirmed: Vec<(super::super::EmbeddedSqlCall, (u32, u32))>,
    order: &[u32],
    promoted: Vec<(PendingRelativeCall, (u32, u32))>,
) -> (Vec<super::super::EmbeddedSqlCall>, Vec<(u32, u32)>) {
    if order.len() != confirmed.len() {
        let mut calls = confirmed;
        calls.extend(
            promoted
                .into_iter()
                .map(|(pending, start)| (pending.call, start)),
        );
        return calls.into_iter().unzip();
    }
    let mut merged = Vec::with_capacity(confirmed.len() + promoted.len());
    let mut confirmed = confirmed.into_iter().zip(order.iter().copied()).peekable();
    let mut pending = promoted.into_iter().peekable();
    loop {
        match (confirmed.peek(), pending.peek()) {
            (Some((_, seq)), Some((call, _))) if call.seq < *seq => {
                let (call, start) = pending.next().expect("peeked pending");
                merged.push((call.call, start));
            }
            (Some(_), _) => merged.push(confirmed.next().expect("peeked confirmed").0),
            (None, Some(_)) => {
                let (call, start) = pending.next().expect("peeked pending");
                merged.push((call.call, start));
            }
            (None, None) => break,
        }
    }
    merged.into_iter().unzip()
}
