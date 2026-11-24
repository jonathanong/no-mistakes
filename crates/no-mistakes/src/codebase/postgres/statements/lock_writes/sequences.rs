use super::{helpers::access, LockWriteFacts, PlaceholderPositions, RowAccess, Segments};
use crate::codebase::postgres::lock_inventory::tokens::RowLockMode;
use sqlparser::ast::{Delete, Expr, FromTable, Query, TableWithJoins};
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
}

pub(super) struct Pending<'a, 'sql> {
    modes: &'a Segments<'sql>,
    locks: Vec<SequenceLock>,
    deletes: Vec<usize>,
}

impl<'a, 'sql> Pending<'a, 'sql> {
    pub(super) fn new(modes: &'a Segments<'sql>) -> Self {
        Self {
            modes,
            locks: Vec::new(),
            deletes: Vec::new(),
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

    pub(super) fn finish(self, facts: &mut LockWriteFacts, first_write: usize) {
        let writes = first_write..facts.writes.len();
        if !self.locks.is_empty() || !writes.is_empty() {
            facts.share_statements.push(ShareStatement {
                locks: self.locks,
                writes,
                deletes: self.deletes,
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
