use crate::codebase::postgres::idents::{object_name_identity, unwrap_expr};
use crate::codebase::postgres::lock_inventory::tokens::Segments;
mod advisory;
mod cte_delete;
mod ddl;
pub(crate) use {
    cte_delete::CteWriteLink,
    ddl::DdlEvent,
    sequences::{SequenceMode, ShareStatement},
};
mod helpers;
mod prelock;
mod row_locks;
mod scope;
mod sequences;
mod settings;
mod trigger_writes;
use super::value::PlaceholderPositions;
pub(crate) use advisory::AdvisoryKey;
use sqlparser::ast::{BinaryOperator, Expr, Query, Statement, TableFactor, Visit, Visitor};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::{ControlFlow, Range},
};
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LockWriteFacts {
    pub ddl_events: Vec<DdlEvent>,
    pub ddl_recovery_complete: bool,
    pub locks: Vec<RowAccess>,
    pub writes: Vec<RowAccess>,
    pub trigger_writes: Vec<crate::codebase::postgres::CatalogTriggerWrite>,
    pub table_locks: Vec<Option<String>>,
    pub transaction_boundary: bool,
    pub relation_identity_changed: bool,
    pub dynamic_setting_name: bool,
    pub advisory_keys: Vec<Vec<AdvisoryKey>>,
    pub linked_cte_locks: BTreeMap<usize, CteWriteLink>,
    pub self_contained_locks: BTreeMap<usize, Range<usize>>,
    pub share_statements: Vec<sequences::ShareStatement>,
    pub prelock: Option<(RowAccess, bool)>,
    pub prelock_attempt: bool,
    pub conditional_write_keys: Vec<RowAccess>,
    pub conditional_write_unique: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RowAccess {
    pub relation: Option<String>,
    pub key: Option<(String, String)>,
    pub recovered_key: Option<(String, (u32, u32))>,
    pub key_identity_unproven: bool,
    pub has_predicate: bool,
}
pub(super) fn collect<const ADVANCED: bool>(
    statements: &[Statement],
    positions: PlaceholderPositions<'_>,
    redundant_prelocks: bool,
    modes: Option<&Segments<'_>>,
) -> LockWriteFacts {
    let mut out = LockWriteFacts::default();
    if let (true, [statement]) = (redundant_prelocks, statements) {
        prelock::collect(statement, &mut out, positions);
    }
    for statement in statements {
        if matches!(statement, Statement::Prepare { .. }) {
            continue;
        }
        let mut executed = Vec::new();
        super::wrappers::walk_executed(statement, &mut executed);
        for statement in executed {
            if ADVANCED {
                ddl::collect(statement, positions, &mut out);
            }
            let (first_lock, first_write) = if ADVANCED {
                (out.locks.len(), out.writes.len())
            } else {
                (0, 0)
            };
            let mut sequence = modes.map(sequences::Pending::new);
            let _ = statement.visit(&mut Collector::<ADVANCED> {
                out: &mut out,
                scopes: Vec::new(),
                cte_depth: 0,
                with_stack: Vec::new(),
                positions,
                sequence: sequence.as_mut(),
                share_top_level: matches!(statement, Statement::Query(_)),
            });
            if ADVANCED {
                scope::mark_self_contained(&mut out, first_lock, first_write);
                if let Some(sequence) = sequence {
                    sequence.finish(&mut out, first_write);
                }
                if out.locks.len() == first_lock + 1
                    && out.self_contained_locks.contains_key(&first_lock)
                {
                    if let Some(link) = cte_delete::linked(statement, positions) {
                        out.linked_cte_locks.insert(first_lock, link);
                    }
                }
            }
        }
    }
    out
}
struct Collector<'a, 'p, 'segments, 'sql, const ADVANCED: bool> {
    out: &'a mut LockWriteFacts,
    scopes: Vec<BTreeSet<String>>,
    cte_depth: usize,
    with_stack: Vec<bool>,
    positions: PlaceholderPositions<'p>,
    sequence: Option<&'a mut sequences::Pending<'segments, 'sql>>,
    share_top_level: bool,
}
impl<const ADVANCED: bool> Visitor for Collector<'_, '_, '_, '_, ADVANCED> {
    type Break = ();
    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        if ADVANCED {
            self.scopes.push(scope::cte_names(query));
        } else {
            self.with_stack.push(query.with.is_some());
            self.cte_depth += usize::from(query.with.is_some());
        }
        if !query.locks.is_empty() {
            if ADVANCED {
                row_locks::collect_advanced(
                    query,
                    self.positions,
                    &self.scopes,
                    &mut self.out.locks,
                    self.sequence.as_deref_mut(),
                    self.share_top_level && self.scopes.len() == 1,
                );
            } else {
                self.out.locks.push(row_locks::collect_legacy(
                    query,
                    self.positions,
                    self.cte_depth == 0,
                ));
            }
        }
        ControlFlow::Continue(())
    }
    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        if ADVANCED {
            self.scopes.pop().expect("balanced SQL query");
        } else {
            self.cte_depth -= usize::from(self.with_stack.pop().expect("balanced SQL query"));
        }
        ControlFlow::Continue(())
    }
    fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<()> {
        trigger_writes::collect(statement, &mut self.out.trigger_writes);
        match statement {
            Statement::CreateTable(table) if table.temporary => {
                self.out.relation_identity_changed = true
            }
            Statement::Commit { .. } | Statement::Rollback { .. } => {
                self.out.transaction_boundary = true
            }
            Statement::Update(update) => {
                if let Some(sequence) = self.sequence.as_deref_mut() {
                    sequence.record_update(
                        self.out.writes.len(),
                        &update.assignments,
                        self.positions,
                    );
                }
                self.out.writes.push(sequences::write(
                    &update.table,
                    update.selection.as_ref(),
                    update.from.is_some(),
                    self.positions,
                ));
            }
            Statement::Delete(delete) => {
                if let Some(row) = sequences::delete(delete, self.positions) {
                    if let Some(sequence) = self.sequence.as_deref_mut() {
                        sequence.record_delete(self.out.writes.len());
                    }
                    self.out.writes.push(row);
                }
            }
            Statement::Lock(lock) => self.out.table_locks.extend(lock.tables.iter().map(|table| {
                (!helpers::generated_name(&table.name, self.positions))
                    .then(|| object_name_identity(&table.name))
            })),
            _ => {}
        }
        ControlFlow::Continue(())
    }
    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        self.out.dynamic_setting_name |= settings::dynamic_setting_name(expr);
        if let Some(keys) = advisory::keys(expr, ADVANCED) {
            self.out.advisory_keys.push(keys);
        }
        ControlFlow::Continue(())
    }
}
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
