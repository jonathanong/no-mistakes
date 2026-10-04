use crate::codebase::postgres::statements::SqlBareRead;
use crate::fx::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;

/// Snapshots share insertion records but retain their preceding-only visibility.
/// Only the query's owning scope appends; enclosing snapshots are read-only.
#[derive(Clone, Default)]
pub(super) struct Names {
    entries: Option<Rc<RefCell<FxHashMap<String, usize>>>>,
    visible: usize,
}

impl Names {
    pub(super) fn insert(&mut self, name: String) {
        let entries = self.entries.get_or_insert_with(Default::default);
        let mut entries = entries.borrow_mut();
        let order = entries.len();
        entries.entry(name).or_insert(order);
        self.visible = entries.len();
    }

    pub(super) fn contains(&self, name: &str) -> bool {
        self.entries.as_ref().is_some_and(|entries| {
            entries
                .borrow()
                .get(name)
                .is_some_and(|order| *order < self.visible)
        })
    }
}

impl Extend<String> for Names {
    fn extend<T: IntoIterator<Item = String>>(&mut self, names: T) {
        for name in names {
            self.insert(name);
        }
    }
}

#[derive(Clone, Default)]
pub(super) struct Tables {
    entries: Option<Rc<RefCell<Vec<String>>>>,
    visible: usize,
}

impl Tables {
    pub(super) fn push(&mut self, table: String) {
        let entries = self.entries.get_or_insert_with(Default::default);
        let mut entries = entries.borrow_mut();
        entries.push(table);
        self.visible = entries.len();
    }
}

#[derive(Clone, Default)]
pub(super) struct Scope {
    pub(super) relations: Names,
    /// The base tables of the level, as SQL names.
    pub(super) tables: Tables,
    /// A relation that is not a base table: a derived table, a function, a CTE.
    pub(super) foreign: bool,
    /// Known projected columns of derived/CTE/function sources in this level.
    pub(super) columns: Names,
}

impl Scope {
    pub(super) fn resolve_reads(&self, reads: &mut Vec<SqlBareRead>) {
        if self.foreign {
            reads.clear();
        } else {
            reads.retain(|read| !self.columns.contains(&read.column));
        }
        let tables = self.tables.entries.as_ref().map(|entries| entries.borrow());
        for read in reads {
            if let Some(tables) = &tables {
                read.tables
                    .extend(tables[..self.tables.visible].iter().cloned());
            }
            read.tables.sort();
            read.tables.dedup();
        }
    }
}

#[cfg(test)]
mod tests;
