use crate::codebase::postgres::{EmbeddedSqlCall, SqlStatementFileFacts};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn group(
    statements: &[SqlStatementFileFacts],
) -> BTreeMap<usize, Vec<&SqlStatementFileFacts>> {
    let mut grouped: BTreeMap<usize, Vec<&SqlStatementFileFacts>> = BTreeMap::new();
    for (index, statement) in statements.iter().filter_map(|statement| {
        statement
            .variant_locations
            .as_ref()
            .map(|locations| (locations.call_index, statement))
    }) {
        grouped.entry(index).or_default().push(statement);
    }
    grouped
}

pub(super) fn unresolved_transactions(
    calls: &[EmbeddedSqlCall],
    grouped: &BTreeMap<usize, Vec<&SqlStatementFileFacts>>,
) -> BTreeSet<u32> {
    calls
        .iter()
        .enumerate()
        .filter(|(index, call)| {
            ((!call.executor_transaction_known && call.caller_transaction_id.is_none())
                || call.is_unanalyzable())
                || grouped.get(index).is_none_or(|versions| {
                    versions.len() != 1
                        || versions.iter().any(|version| {
                            (version.parse_failed && !version.lock_writes.ddl_recovery_complete)
                                || version
                                    .lock_writes
                                    .writes
                                    .iter()
                                    .any(|write| write.relation.is_none())
                                || version.lock_writes.transaction_boundary
                                || version.lock_writes.relation_identity_changed
                                || version
                                    .setting_uses
                                    .iter()
                                    .any(|setting| setting.name == "search_path")
                                || (!call.execution_unconditional
                                    && !version.lock_writes.writes.is_empty())
                        })
                })
        })
        .filter_map(|(_, call)| call.transaction_id.or(call.caller_transaction_id))
        .collect()
}
