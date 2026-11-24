use super::{
    helpers::{access, generated_name},
    LockWriteFacts, PlaceholderPositions, RowAccess, Segments,
};
use crate::codebase::postgres::idents::{ident_key, object_name_ident, unwrap_expr};
use crate::codebase::postgres::lock_inventory::tokens::RowLockMode;
use sqlparser::ast::{
    Assignment, AssignmentTarget, Delete, Expr, FromTable, Query, TableWithJoins,
};
use std::collections::BTreeMap;
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SequenceMode {
    Share,
    KeyShare,
    Update,
    UnknownShare,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SequenceLock {
    pub mode: SequenceMode,
    pub row: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShareStatement {
    pub locks: Vec<SequenceLock>,
    pub writes: Range<usize>,
    pub deletes: Vec<usize>,
    pub update_columns: BTreeMap<usize, Vec<String>>,
}

pub(super) struct Pending<'a, 'sql> {
    modes: &'a Segments<'sql>,
    locks: Vec<SequenceLock>,
    deletes: Vec<usize>,
    update_columns: BTreeMap<usize, Vec<String>>,
}

impl<'a, 'sql> Pending<'a, 'sql> {
    pub(super) fn new(modes: &'a Segments<'sql>) -> Self {
        Self {
            modes,
            locks: Vec::new(),
            deletes: Vec::new(),
            update_columns: BTreeMap::new(),
        }
    }

    pub(super) fn modes(&self, query: &Query) -> Option<Vec<RowLockMode>> {
        self.modes.row_lock_modes(query)
    }

    pub(super) fn lock(&mut self, mode: SequenceMode, row: Option<usize>) {
        self.locks.push(SequenceLock { mode, row });
    }

    pub(super) fn record_delete(&mut self, index: usize) {
        self.deletes.push(index);
    }

    pub(super) fn record_update(
        &mut self,
        index: usize,
        assignments: &[Assignment],
        positions: PlaceholderPositions<'_>,
    ) {
        let columns = assignments
            .iter()
            .flat_map(|assignment| {
                let targets = match &assignment.target {
                    AssignmentTarget::ColumnName(name) => std::slice::from_ref(name),
                    AssignmentTarget::Tuple(names) => names.as_slice(),
                };
                targets.iter().filter_map(move |name| {
                    if name.0.len() != 1 || generated_name(name, positions) {
                        return None;
                    }
                    let column = ident_key(object_name_ident(name)?);
                    if matches!((&assignment.target, unwrap_expr(&assignment.value)),
                        (AssignmentTarget::ColumnName(_), Expr::Identifier(source))
                            if ident_key(source) == column)
                    {
                        return None;
                    }
                    Some(column)
                })
            })
            .collect::<Vec<_>>();
        if !columns.is_empty() {
            self.update_columns.insert(index, columns);
        }
    }

    pub(super) fn finish(self, facts: &mut LockWriteFacts, first_write: usize) {
        let writes = first_write..facts.writes.len();
        if !self.locks.is_empty() || !writes.is_empty() {
            facts.share_statements.push(ShareStatement {
                locks: self.locks,
                writes,
                deletes: self.deletes,
                update_columns: self.update_columns,
            });
        }
    }
}

pub(super) fn delete(delete: &Delete, positions: PlaceholderPositions<'_>) -> Option<RowAccess> {
    let (FromTable::WithFromKeyword(from) | FromTable::WithoutKeyword(from)) = &delete.from;
    let [table] = from.as_slice() else {
        return None;
    };
    Some(write(
        table,
        delete.selection.as_ref(),
        delete.using.is_some(),
        positions,
    ))
}

pub(super) fn write(
    table: &TableWithJoins,
    predicate: Option<&Expr>,
    joined: bool,
    positions: PlaceholderPositions<'_>,
) -> RowAccess {
    let mut row = access(&table.relation, predicate, positions);
    if joined || !table.joins.is_empty() {
        row.key = None;
        row.recovered_key = None;
        row.has_predicate = true;
    }
    row
}
