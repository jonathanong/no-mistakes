use super::*;

mod potential;
mod rows_by_relation;

use crate::codebase::postgres::statements::lock_writes::{RowAccess, SequenceMode};
use rows_by_relation::{EarlierUpdates, LaterWrites};

pub(super) fn scan(
    output: &mut Output<'_>,
    file: &str,
    source: &str,
    calls: &[EmbeddedSqlCall],
    grouped: &BTreeMap<usize, Vec<&SqlStatementFileFacts>>,
    unresolved: &BTreeSet<u32>,
) {
    let mut ordered = Vec::new();
    let mut uncertain_writes = BTreeMap::<_, relations::Writes<'_>>::new();
    let mut uncertain_deletes = BTreeMap::<_, relations::Writes<'_>>::new();
    let candidates = variants::groups(calls, grouped);
    let potential = potential::PotentialWrites::collect(candidates.clone());
    for (call, versions) in candidates {
        let proven = match versions {
            [facts] if !facts.parse_failed => {
                ordered_transaction(call, unresolved).map(|tx| (tx, *facts))
            }
            _ => None,
        };
        if let Some((transaction, facts)) = proven {
            if !facts.lock_writes.share_statements.is_empty() {
                ordered.push((transaction, call, facts));
            }
            continue;
        }
        if let Some(transaction) = transaction_scope(call) {
            for facts in versions.iter().filter(|facts| !facts.parse_failed) {
                let access = &facts.lock_writes;
                uncertain_writes.entry(transaction).or_default().extend(
                    access
                        .writes
                        .iter()
                        .filter_map(|row| row.relation.as_deref()),
                );
                uncertain_deletes.entry(transaction).or_default().extend(
                    access
                        .share_statements
                        .iter()
                        .flat_map(|statement| &statement.deletes)
                        .filter_map(|&index| access.writes[index].relation.as_deref()),
                );
            }
        }
        if !known_autocommit(call)
            && versions
                .iter()
                .any(|facts| !facts.parse_failed && potential.may_follow_lock(call, facts))
        {
            output.unresolved(
                &Site { file, source, call },
                "share lock transaction or statement order is unresolved; use a proven transaction executor and await the lock and write",
            );
        }
    }
    ordered.sort_unstable_by_key(|(transaction, call, _)| (*transaction, call.execution_range.1));
    let mut current = None;
    let mut written = LaterWrites::default();
    let mut deleted = LaterWrites::default();
    let earlier = EarlierUpdates::collect(&ordered);
    for (transaction, call, facts) in ordered.into_iter().rev() {
        if current != Some(transaction) {
            current = Some(transaction);
            written = LaterWrites::default();
            deleted = LaterWrites::default();
        }
        let access = &facts.lock_writes;
        let mut reported = BTreeSet::new();
        for (ordinal, statement) in access.share_statements.iter().enumerate().rev() {
            let site = Site { file, source, call };
            for lock in &statement.locks {
                if lock.mode == SequenceMode::Update {
                    continue;
                }
                let row = lock.row.and_then(|index| access.locks.get(index));
                let relation = row.and_then(|row| row.relation.as_deref());
                if reported.contains(&relation) {
                    continue;
                }
                let deletes_only = lock.mode == SequenceMode::KeyShare;
                let (later, uncertain) = if deletes_only {
                    (&deleted, uncertain_deletes.get(&transaction))
                } else {
                    (&written, uncertain_writes.get(&transaction))
                };
                if let (Some(relation), Some(row)) = (relation, row) {
                    if earlier.contains(transaction, call, ordinal, row) {
                        continue;
                    }
                    if later.matches(relation, row, call) && lock.mode != SequenceMode::UnknownShare
                    {
                        output.emit(
                            &site,
                            super::super::super::SHARE_LOCK_THEN_WRITE,
                            Some(relation),
                            None,
                            "share lock precedes a conflicting write on the same row or relation-wide target; take a write-strength lock up front or use a conditional write",
                        );
                    } else if later.uncertain(relation, row, call)
                        || (lock.mode == SequenceMode::UnknownShare
                            && later.relations.contains(relation))
                    {
                        output.unresolved(
                            &site,
                            "cannot prove the share lock mode or common row key; use explicit lock clauses and comparable row predicates",
                        );
                    } else if uncertain.is_some_and(|writes| writes.contains(relation)) {
                        output.unresolved(
                            &site,
                            "cannot prove whether the write executes after this share lock; await the lock and write directly in the same transaction",
                        );
                    } else if later.relations.ambiguous(relation)
                        || uncertain.is_some_and(|writes| writes.ambiguous(relation))
                    {
                        output.unresolved(
                            &site,
                            "cannot prove the share lock and write target the same relation; qualify both relation names consistently",
                        );
                    } else {
                        continue;
                    }
                } else if !later.relations.is_empty()
                    || uncertain.is_some_and(|writes| !writes.is_empty())
                    || potential.may_follow(call, None, deletes_only)
                {
                    output.unresolved(
                        &site,
                        "cannot prove shared lock execution on a base relation; use a top-level lock and an explicit relation or FOR SHARE OF alias",
                    );
                } else {
                    continue;
                }
                reported.insert(relation);
            }
            for index in statement.writes.clone() {
                let row = &access.writes[index];
                written.insert(row, call);
                if statement.deletes.binary_search(&index).is_ok() {
                    deleted.insert(row, call);
                }
            }
        }
    }
}
