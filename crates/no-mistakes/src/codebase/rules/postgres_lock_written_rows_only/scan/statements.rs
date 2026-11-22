use crate::codebase::postgres::{
    idents::relation_suffix_key, statements::lock_writes::DdlEvent, EmbeddedSqlCall,
    SqlStatementFileFacts,
};
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

#[derive(Default)]
pub(super) struct CatalogInvalidations<'a> {
    relations: BTreeSet<&'a str>,
    uncertain_scopes: BTreeSet<u32>,
}

impl<'a> CatalogInvalidations<'a> {
    pub(super) fn collect(
        calls: &[EmbeddedSqlCall],
        grouped: &BTreeMap<usize, Vec<&'a SqlStatementFileFacts>>,
    ) -> Self {
        let mut invalidations = Self::default();
        for (index, call) in calls.iter().enumerate() {
            let versions = grouped.get(&index).map(Vec::as_slice).unwrap_or_default();
            if call.is_unanalyzable()
                || versions.len() != 1
                || versions.iter().any(|facts| {
                    facts.parse_failed
                        || facts.lock_writes.relation_identity_changed
                        || facts.lock_writes.dynamic_setting_name
                        || facts
                            .setting_uses
                            .iter()
                            .any(|setting| setting.name == "search_path")
                })
            {
                invalidations.uncertain_scopes.insert(call.execution_scope);
            }
            for change in versions
                .iter()
                .flat_map(|facts| &facts.lock_writes.ddl_events)
            {
                if let DdlEvent::Change(relation) = change {
                    invalidations
                        .relations
                        .insert(relation_suffix_key(relation));
                }
            }
        }
        invalidations
    }

    pub(super) fn permits(&self, call: &EmbeddedSqlCall, relation: Option<&str>) -> bool {
        !self.uncertain_scopes.contains(&call.execution_scope)
            && relation
                .is_some_and(|relation| !self.relations.contains(relation_suffix_key(relation)))
    }
}
