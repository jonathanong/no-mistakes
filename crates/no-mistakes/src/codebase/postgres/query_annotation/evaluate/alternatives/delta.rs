//! Arm joins visit the union of written locations, never an incoming table.
mod apply;
mod sentinels;
mod value;
use super::super::{
    transactions::{Cell, Delta, Saved},
    Evaluator,
};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};
#[derive(Default)]
pub(super) struct Join {
    entries: FxHashMap<Cell, Entry>,
    arms: usize,
    pins: Vec<FxHashSet<u64>>,
    shapes: FxHashSet<u64>,
    initial_sentinels: FxHashMap<u64, bool>,
    pub changed_builders: FxHashSet<u64>,
}
struct Entry {
    initial: Saved,
    joined: Saved,
}
impl Join {
    pub(super) fn add(&mut self, delta: Delta, whole_sql: bool) {
        self.pins.push(delta.pins);
        let changes = delta.changes;
        self.track_sentinels(&changes);
        let before_builders = value::builders(&changes);
        let mut present = FxHashSet::default();
        if !whole_sql {
            for change in &changes {
                value::changes(change, &mut self.changed_builders);
            }
        }
        let mut visited = 0_u64;
        for mut change in changes {
            visited += 1;
            if change.before == change.after {
                continue;
            }
            if let Cell::Builder(id) = &change.cell {
                if change.before == Saved::Value(None) {
                    if let Some(value) = before_builders.get(id) {
                        change.before = Saved::Value(Some(value.clone()));
                    }
                }
            }
            present.insert(change.cell.clone());
            match self.entries.entry(change.cell.clone()) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    let joined = if self.arms == 0 {
                        value::first(&change.cell, &change.before, &change.after)
                    } else {
                        value::join(
                            &change.cell,
                            &change.before,
                            &change.before,
                            &change.after,
                            whole_sql,
                            &mut self.shapes,
                        )
                    };
                    entry.insert(Entry {
                        initial: change.before,
                        joined,
                    });
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    let entry = entry.get_mut();
                    entry.joined = value::join(
                        &change.cell,
                        &entry.initial,
                        &entry.joined,
                        &change.after,
                        whole_sql,
                        &mut self.shapes,
                    );
                }
            }
        }
        for (cell, entry) in &mut self.entries {
            visited += 1;
            if !present.contains(cell) {
                entry.joined = value::join(
                    cell,
                    &entry.initial,
                    &entry.joined,
                    &entry.initial,
                    whole_sql,
                    &mut self.shapes,
                );
            }
        }
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.join_delta_cells_visited", visited);
        }
        self.arms += 1;
    }
    pub(super) fn release_pins<F: Fn(&str, &Path) -> Option<PathBuf>>(
        &mut self,
        evaluator: &mut Evaluator<'_, F>,
    ) {
        for pins in self.pins.drain(..) {
            evaluator.release_delta_pins(pins);
        }
    }
    pub(super) fn remembered_roots<F: Fn(&str, &Path) -> Option<PathBuf>>(
        &self,
        evaluator: &Evaluator<'_, F>,
    ) -> Vec<super::super::temporaries::TransientRoot> {
        let mut roots = Vec::new();
        for cell in self.entries.keys() {
            evaluator.append_cell_roots(cell, &mut roots);
        }
        roots
    }
}
