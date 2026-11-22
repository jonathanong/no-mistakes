use super::*;

mod potential;

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
            uncertain_writes.entry(transaction).or_default().extend(
                versions
                    .iter()
                    .filter(|facts| !facts.parse_failed)
                    .flat_map(|facts| &facts.lock_writes.writes)
                    .filter_map(|write| write.relation.as_deref()),
            );
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
                } else if !written.is_empty()
                    || uncertain.is_some_and(|writes| !writes.is_empty())
                    || potential.may_follow(call, None)
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
            written.extend(
                access.writes[statement.writes.clone()]
                    .iter()
                    .filter_map(|row| row.relation.as_deref()),
            );
        }
    }
}
