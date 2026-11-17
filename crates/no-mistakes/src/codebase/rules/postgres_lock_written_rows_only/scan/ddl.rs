use crate::codebase::postgres::{
    statements::lock_writes::DdlEvent, EmbeddedSqlCall, SchemaCatalog, SqlStatementFileFacts,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn exemptions(
    calls: &[EmbeddedSqlCall],
    grouped: &BTreeMap<usize, Vec<&SqlStatementFileFacts>>,
    unresolved: &BTreeSet<u32>,
    catalog: Option<&SchemaCatalog>,
) -> BTreeSet<(usize, usize)> {
    let overlapping = overlapping_transactions(calls);
    let mut transactions: BTreeMap<(u32, u32), Vec<(usize, &DdlEvent)>> = BTreeMap::new();
    for (index, call) in calls.iter().enumerate() {
        let Some(id) = call
            .transaction_id
            .filter(|id| !unresolved.contains(id) && !overlapping.contains(id))
        else {
            continue;
        };
        if call.helper_definition
            || !call.execution_awaited
            || !call.execution_unconditional
            || !call.executor_transaction_known
        {
            continue;
        }
        if let Some(versions) = grouped.get(&index).filter(|versions| versions.len() == 1) {
            transactions
                .entry((id, call.execution_scope))
                .or_default()
                .extend(
                    versions[0]
                        .lock_writes
                        .ddl_events
                        .iter()
                        .map(|event| (index, event)),
                );
        }
    }
    let mut exemptions = BTreeSet::new();
    for events in transactions.values() {
        let mut locks = Vec::new();
        let mut relationships = Vec::new();
        for (index, event) in events {
            match event {
                DdlEvent::Related(left, right) => relationships.push((left, right)),
                DdlEvent::Lock(lock) => {
                    if let Some(relation) =
                        grouped[index][0].lock_writes.table_locks[*lock].as_deref()
                    {
                        locks.push((*index, *lock, relation, false));
                    }
                }
                DdlEvent::Read(relation) => {
                    for (_, _, locked, read) in &mut locks {
                        *read |= same_relation(locked, relation, catalog);
                    }
                }
                DdlEvent::Change(target) => {
                    for (index, lock, relation, read) in &locks {
                        if *read
                            || same_relation(relation, target, catalog)
                            || relationships.iter().any(|(left, right)| {
                                (*left == *relation && *right == target)
                                    || (*right == *relation && *left == target)
                            })
                            || catalog.is_some_and(|catalog| related(catalog, relation, target))
                        {
                            exemptions.insert((*index, *lock));
                        }
                    }
                }
            }
        }
    }
    exemptions
}
fn same_relation(left: &str, right: &str, catalog: Option<&SchemaCatalog>) -> bool {
    if left == right {
        return true;
    }
    catalog.is_some_and(|catalog| catalog.same_relation(left, right))
}
fn related(catalog: &SchemaCatalog, left: &str, right: &str) -> bool {
    let links = |table: &crate::codebase::postgres::CatalogTable, target: &str| {
        table
            .partition_of
            .as_deref()
            .is_some_and(|parent| same_relation(parent, target, Some(catalog)))
            || table
                .inherited_from
                .iter()
                .any(|parent| same_relation(parent, target, Some(catalog)))
            || table
                .foreign_keys
                .iter()
                .any(|fk| same_relation(&fk.referenced_table, target, Some(catalog)))
    };
    catalog
        .relation(left)
        .is_some_and(|table| links(table, right))
        || catalog
            .relation(right)
            .is_some_and(|table| links(table, left))
}

fn overlapping_transactions(calls: &[EmbeddedSqlCall]) -> BTreeSet<u32> {
    let mut ranges: BTreeMap<(u32, u32), Vec<(u32, u32)>> = BTreeMap::new();
    for call in calls {
        if let Some(transaction) = call.transaction_id {
            if !call.helper_definition {
                ranges
                    .entry((transaction, call.execution_scope))
                    .or_default()
                    .push(call.execution_range);
            }
        }
    }
    ranges
        .into_iter()
        .filter_map(|((transaction, _), mut spans)| {
            spans.sort_unstable();
            spans
                .windows(2)
                .any(|pair| pair[0].1 > pair[1].0)
                .then_some(transaction)
        })
        .collect()
}
