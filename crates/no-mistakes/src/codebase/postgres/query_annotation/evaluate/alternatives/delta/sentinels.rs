//! Preserve main's contagious singleton failure without scanning untouched objects.
use super::super::super::{transactions::Change, Value};
use super::*;
impl Join {
    pub(super) fn track_sentinels(&mut self, changes: &[Change]) {
        let lengths = changes
            .iter()
            .filter_map(|change| {
                if let (Cell::Length(id), Saved::Length(before), Saved::Length(after)) =
                    (&change.cell, &change.before, &change.after)
                {
                    Some((*id, (*before, *after)))
                } else {
                    None
                }
            })
            .collect::<FxHashMap<_, _>>();
        let failure = Saved::Value(Some(Value::Failure("conditional-collection-shape")));
        let mut present = FxHashSet::default();
        for change in changes {
            if let Cell::Dense(id, 0) = change.cell {
                let (before_length, after_length) = lengths[&id];
                let initial = before_length == 1 && change.before == failure;
                let current = after_length == 1 && change.after == failure;
                if current || (self.arms > 0 && initial) {
                    self.shapes.insert(id);
                }
                self.initial_sentinels.entry(id).or_insert(initial);
                present.insert(id);
            }
        }
        for (id, initial) in &self.initial_sentinels {
            if *initial && !present.contains(id) {
                self.shapes.insert(*id);
            }
        }
    }
}
