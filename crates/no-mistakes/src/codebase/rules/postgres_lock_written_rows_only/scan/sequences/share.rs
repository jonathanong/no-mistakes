use super::*;

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
    let candidates = complete_calls(calls, grouped)
        .filter(|(_, facts)| !facts.lock_writes.share_statements.is_empty());
    for (call, facts) in candidates {
        let proven_order = ordered_transaction(call, unresolved).is_some();
        if let Some(transaction) = transaction_scope(call) {
            if proven_order {
                ordered.push((transaction, call, facts));
            } else {
                uncertain_writes.entry(transaction).or_default().extend(
                    facts
                        .lock_writes
                        .writes
                        .iter()
                        .filter_map(|write| write.relation.as_deref()),
                );
            }
        }
        if !proven_order
            && !known_autocommit(call)
            && facts
                .lock_writes
                .share_statements
                .iter()
                .any(|statement| !statement.locks.is_empty())
        {
            output.unresolved(
                &Site { file, source, call },
                "share lock transaction or statement order is unresolved; use a proven transaction executor and await the lock and write",
            );
        }
    }
    ordered.sort_unstable_by_key(|(transaction, call, _)| (*transaction, call.execution_range.1));
    let mut current = None;
    let mut written = relations::Writes::default();
    for (transaction, call, facts) in ordered.into_iter().rev() {
        if current != Some(transaction) {
            current = Some(transaction);
            written.clear();
        }
        let access = &facts.lock_writes;
        let uncertain = uncertain_writes.get(&transaction);
        let mut reported = BTreeSet::new();
        for statement in access.share_statements.iter().rev() {
            let site = Site { file, source, call };
            for relation in &statement.locks {
                let relation = relation.as_deref();
                if reported.contains(&relation) {
                    continue;
                }
                if let Some(relation) = relation {
                    if written.contains(relation) {
                        output.emit(
                            &site,
                            super::super::super::SHARE_LOCK_THEN_WRITE,
                            Some(relation),
                            None,
                            "share lock precedes an update or delete of this relation in the same transaction; take a write-strength lock up front or use a conditional write",
                        );
                    } else if uncertain.is_some_and(|writes| writes.contains(relation)) {
                        output.unresolved(
                            &site,
                            "cannot prove whether the write executes after this share lock; await the lock and write directly in the same transaction",
                        );
                    } else if written.ambiguous(relation)
                        || uncertain.is_some_and(|writes| writes.ambiguous(relation))
                    {
                        output.unresolved(
                            &site,
                            "cannot prove the share lock and write target the same relation; qualify both relation names consistently",
                        );
                    } else {
                        continue;
                    }
                } else {
                    output.unresolved(
                        &site,
                        "cannot prove shared lock execution on a base relation; use a top-level lock and an explicit relation or FOR SHARE OF alias",
                    );
                }
                reported.insert(relation);
            }
            written.extend(
                access.writes[statement.writes.clone()]
                    .iter()
                    .filter_map(|row| row.relation.as_deref()),
            );
        }
    }
}
