use crate::codebase::postgres::{EmbeddedSqlCall, SqlStatementFileFacts};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn trigger_disabled_transactions(
    calls: &[EmbeddedSqlCall],
    grouped: &BTreeMap<usize, Vec<&SqlStatementFileFacts>>,
) -> BTreeSet<u32> {
    calls
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            grouped.get(index).is_some_and(|versions| {
                versions.iter().any(|version| {
                    version.setting_uses.iter().any(|setting| {
                        setting
                            .name
                            .eq_ignore_ascii_case("session_replication_role")
                    })
                })
            })
        })
        .filter_map(|(_, call)| call.transaction_id)
        .collect()
}
