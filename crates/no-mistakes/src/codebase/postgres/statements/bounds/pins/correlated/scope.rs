use crate::codebase::postgres::statements::SqlBareRead;
use std::collections::BTreeSet;

#[derive(Clone, Default)]
pub(super) struct Scope {
    pub(super) relations: BTreeSet<String>,
    /// The base tables of the level, as SQL names.
    pub(super) tables: Vec<String>,
    /// A relation that is not a base table: a derived table, a function, a CTE.
    pub(super) foreign: bool,
    /// Known projected columns of derived/CTE/function sources in this level.
    pub(super) columns: BTreeSet<String>,
}

impl Scope {
    pub(super) fn resolve_reads(&self, reads: &mut Vec<SqlBareRead>) {
        if self.foreign {
            reads.clear();
        } else {
            reads.retain(|read| !self.columns.contains(&read.column));
        }
        for read in reads {
            read.tables.extend(self.tables.iter().cloned());
            read.tables.sort();
            read.tables.dedup();
        }
    }
}
