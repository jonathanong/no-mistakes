use super::{progress, Arena, Value};
use crate::fx::{FxHashMap, FxHashSet};

type Prefixes<'a> = FxHashMap<u64, (&'a str, bool)>;

pub(super) fn collect<'a, const OBSERVED: bool>(
    value: &'a Value,
    arena: Arena<'a>,
    definite: &FxHashSet<(u64, usize)>,
    work: &mut Option<progress::Walk>,
) -> Prefixes<'a> {
    let mut walk = Walk::<OBSERVED> {
        arena,
        definite,
        work,
        visited: FxHashSet::default(),
        found: FxHashMap::default(),
        completed: FxHashMap::default(),
    };
    walk.visit(value, None);
    walk.found
}

struct Walk<'a, 'w, const OBSERVED: bool> {
    arena: Arena<'a>,
    definite: &'w FxHashSet<(u64, usize)>,
    work: &'w mut Option<progress::Walk>,
    visited: FxHashSet<u64>,
    found: Prefixes<'a>,
    completed: FxHashMap<usize, Prefixes<'a>>,
}

impl<'a, const OBSERVED: bool> Walk<'a, '_, OBSERVED> {
    fn visit(&mut self, value: &'a Value, mut direct: Option<&mut Prefixes<'a>>) {
        // Specialization removes the per-node diagnostic branch when disabled.
        if OBSERVED {
            if let Some(work) = self.work.as_mut() {
                work.visit(value);
            }
        }
        match value {
            Value::Prefix(text, complete, Some(id)) => {
                self.found.insert(*id, (text, *complete));
                if let Some(direct) = direct {
                    direct.insert(*id, (text, *complete));
                }
            }
            Value::Promise(value) | Value::Evaluated(value, _) => self.visit(value, direct),
            Value::Aggregate(values)
            | Value::References(values)
            | Value::Object(values)
            | Value::Record(values, _)
            | Value::Joined(values)
            | Value::Possible(values) => {
                if !values.contains_reference() {
                    return;
                }
                // A uniquely owned chain cannot be revisited. Forward writes
                // directly instead of retaining overlapping ancestor summaries.
                if !values.is_shared() {
                    for child in values {
                        self.visit(child, direct.as_deref_mut());
                    }
                    return;
                }
                let identity = values.identity() as usize;
                if let Some(summary) = self.completed.get(&identity) {
                    // All argument edges in a completed occurrence were visited.
                    // Replay direct writes: skipping them would change last-writer order.
                    self.found
                        .extend(summary.iter().map(|(id, value)| (*id, *value)));
                    if let Some(direct) = direct {
                        direct.extend(summary.iter().map(|(id, value)| (*id, *value)));
                    }
                    return;
                }
                let mut summary = FxHashMap::default();
                for child in values {
                    self.visit(child, Some(&mut summary));
                }
                if let Some(direct) = direct {
                    direct.extend(summary.iter().map(|(id, value)| (*id, *value)));
                }
                // Publish only after completion: mutable argument cycles can
                // reenter this record before its later children have been visited.
                self.completed.insert(identity, summary);
            }
            Value::Arguments(id) | Value::Array(id) if self.visited.insert(*id) => {
                for (index, child) in self.arena.objects[id].iter().enumerate().chain(
                    self.arena
                        .extras
                        .get(id)
                        .into_iter()
                        .flat_map(|slots| slots.iter().map(|(index, value)| (*index, value))),
                ) {
                    if !self.definite.contains(&(*id, index)) {
                        // Argument-derived writes are visited once and must not
                        // enter a replayable immutable-container summary.
                        self.visit(child, None);
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
