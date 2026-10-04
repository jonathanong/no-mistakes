use crate::codebase::postgres::statements::{SqlBareRead, SqlQualifiedScope};
use crate::fx::FxHashMap;
use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

/// Snapshots share insertion records but retain their preceding-only visibility.
/// Only the query's owning scope appends; enclosing snapshots are read-only.
#[derive(Clone, Default)]
pub(super) struct Names<K = String> {
    entries: Option<Rc<RefCell<FxHashMap<K, usize>>>>,
    visible: usize,
}

impl<K: Eq + Hash> Names<K> {
    pub(super) fn insert(&mut self, name: K) {
        let entries = self.entries.get_or_insert_with(Default::default);
        let mut entries = entries.borrow_mut();
        let order = entries.len();
        entries.entry(name).or_insert(order);
        self.visible = entries.len();
    }

    pub(super) fn contains<Q: Eq + Hash + ?Sized>(&self, name: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
    {
        self.entries.as_ref().is_some_and(|entries| {
            entries
                .borrow()
                .get(name)
                .is_some_and(|order| *order < self.visible)
        })
    }
}

impl<K: Eq + Hash> Extend<K> for Names<K> {
    fn extend<T: IntoIterator<Item = K>>(&mut self, names: T) {
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

    fn visible(&self) -> Vec<String> {
        self.entries
            .as_ref()
            .map(|entries| entries.borrow()[..self.visible].to_vec())
            .unwrap_or_default()
    }
}

#[derive(Clone, Default)]
pub(super) struct Scope {
    /// Identifier components distinguish quoted dots from path separators.
    pub(super) relations: Names<Vec<String>>,
    pub(super) whole_rows: Names,
    /// The base tables of the level, as SQL names.
    pub(super) tables: Tables,
    /// Base names visible to qualified SQL references. An explicit table alias removes its
    /// base name from this lexical namespace while `tables` still helps resolve bare columns.
    pub(super) qualified_tables: Tables,
    /// A relation that is not a base table: a derived table, a function, a CTE.
    pub(super) foreign: bool,
    /// Known projected columns of derived/CTE/function sources in this level.
    pub(super) columns: Names,
}

impl Scope {
    pub(super) fn qualified_candidates(&self) -> SqlQualifiedScope {
        SqlQualifiedScope {
            tables: self.qualified_tables.visible(),
            unknown: self.foreign,
        }
    }

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
