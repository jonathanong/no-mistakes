use super::{progress, Arena, Value};
use crate::fx::{FxHashMap, FxHashSet};

type Prefixes<'a> = FxHashMap<u64, (&'a str, bool)>;

#[derive(Clone, Copy)]
enum Shape<'a> {
    Empty,
    One(u64, (&'a str, bool)),
    Many,
}
impl<'a> Shape<'a> {
    fn merge(&mut self, other: Self) {
        *self = match (*self, other) {
            (current, Self::Empty) => current,
            (Self::Empty, next) => next,
            (Self::One(id, _), Self::One(other, value)) if id == other => Self::One(id, value),
            _ => Self::Many,
        };
    }
}

struct Summary<'a> {
    shape: Shape<'a>,
    full: Option<Prefixes<'a>>,
}

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
    completed: FxHashMap<usize, Summary<'a>>,
}

impl<'a, const OBSERVED: bool> Walk<'a, '_, OBSERVED> {
    fn visit(&mut self, value: &'a Value, mut direct: Option<&mut Prefixes<'a>>) -> Shape<'a> {
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
                return Shape::One(*id, (text, *complete));
            }
            Value::Promise(value) | Value::Evaluated(value, _) => return self.visit(value, direct),
            Value::Aggregate(values)
            | Value::References(values)
            | Value::Object(values)
            | Value::Record(values, _)
            | Value::Joined(values)
            | Value::Possible(values) => {
                if !values.contains_reference() {
                    return Shape::Empty;
                }
                let identity = values.identity() as usize;
                // Arc::strong_count is a shortcut for this single-threaded walk;
                // parallel walks must revisit that ownership assumption.
                let shared = values.is_shared();
                let previous = if shared {
                    self.completed.get(&identity)
                } else {
                    None
                };
                if let Some(summary) = previous {
                    let entries = if let Some(full) = &summary.full {
                        self.found
                            .extend(full.iter().map(|(id, value)| (*id, *value)));
                        if let Some(direct) = direct.as_deref_mut() {
                            direct.extend(full.iter().map(|(id, value)| (*id, *value)));
                        }
                        Some(full.len())
                    } else {
                        match summary.shape {
                            Shape::Empty => Some(0),
                            Shape::One(id, prefix) => {
                                self.found.insert(id, prefix);
                                if let Some(direct) = direct.as_deref_mut() {
                                    direct.insert(id, prefix);
                                }
                                Some(1)
                            }
                            Shape::Many => None,
                        }
                    };
                    if let Some(entries) = entries {
                        if OBSERVED {
                            if let Some(work) = self.work.as_mut() {
                                work.replay(entries);
                            }
                        }
                        return summary.shape;
                    }
                }
                // Snapshot aliases alone do not justify overlapping descendant
                // maps. Empty/singleton summaries are inline; larger summaries
                // require a completed occurrence to have actually been reused.
                let mut full = previous.map(|_| FxHashMap::default());
                let mut shape = Shape::Empty;
                for child in values {
                    let output = if full.is_some() {
                        full.as_mut()
                    } else {
                        direct.as_deref_mut()
                    };
                    shape.merge(self.visit(child, output));
                }
                if let (Some(direct), Some(full)) = (direct, &full) {
                    direct.extend(full.iter().map(|(id, value)| (*id, *value)));
                }
                if shared {
                    if OBSERVED {
                        if let Some(work) = self.work.as_mut() {
                            work.summary(full.as_ref().map_or_else(
                                || usize::from(matches!(shape, Shape::One(..))),
                                FxHashMap::len,
                            ));
                        }
                    }
                    // Argument cycles can reenter this container: publish only
                    // after every direct child has completed.
                    self.completed.insert(identity, Summary { shape, full });
                }
                return shape;
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
                        // Argument writes occur once, outside replayable summaries.
                        self.visit(child, None);
                    }
                }
            }
            _ => {}
        }
        Shape::Empty
    }
}

#[cfg(test)]
mod tests;
