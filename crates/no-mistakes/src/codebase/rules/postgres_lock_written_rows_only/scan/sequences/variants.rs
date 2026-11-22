use super::*;

pub(super) fn groups<'a>(
    calls: &'a [EmbeddedSqlCall],
    grouped: &'a BTreeMap<usize, Vec<&'a SqlStatementFileFacts>>,
) -> impl Iterator<Item = (&'a EmbeddedSqlCall, &'a [&'a SqlStatementFileFacts])> + Clone {
    calls.iter().enumerate().filter_map(|(index, call)| {
        (!call.helper_definition && !call.is_unanalyzable())
            .then_some((call, grouped.get(&index)?.as_slice()))
    })
}

pub(super) fn scan_redundant(
    output: &mut Output<'_>,
    file: &str,
    source: &str,
    calls: &[EmbeddedSqlCall],
    grouped: &BTreeMap<usize, Vec<&SqlStatementFileFacts>>,
) {
    let candidates = groups(calls, grouped).filter(|(call, _)| !known_autocommit(call));
    let writes: BTreeMap<_, _> = candidates
        .clone()
        .filter(|(_, versions)| {
            versions.iter().any(|facts| {
                !facts.parse_failed && !facts.lock_writes.conditional_write_keys.is_empty()
            })
        })
        .map(|(call, versions)| (call.execution_range.0, (call, versions)))
        .collect();
    for (lock_call, locks) in candidates {
        let Some((write_call, writes)) = lock_call
            .next_discarded_query
            .and_then(|next| writes.get(&next))
        else {
            continue;
        };
        if locks.len() == 1 && writes.len() == 1 {
            continue;
        }
        if matches!((transaction_scope(lock_call), transaction_scope(write_call)),
            (Some(left), Some(right)) if left != right)
        {
            continue;
        }
        if locks.iter().any(|lock| {
            !lock.parse_failed
                && lock.lock_writes.prelock_attempt
                && writes
                    .iter()
                    .filter(|write| !write.parse_failed)
                    .any(|write| {
                        write
                            .lock_writes
                            .conditional_write_keys
                            .iter()
                            .any(|write| {
                                lock.lock_writes.locks.iter().any(|row| {
                                    relations::same(
                                        row.relation.as_deref(),
                                        write.relation.as_deref(),
                                    ) != Some(false)
                                        && rows::compare_keys(row, lock_call, write, write_call)
                                            != Some(false)
                                })
                            })
                    })
        }) {
            output.unresolved(
                &Site { file, source, call: lock_call },
                "cannot prove the redundant pre-lock sequence across SQL variants; use one fixed lock and write statement",
            );
        }
    }
}
