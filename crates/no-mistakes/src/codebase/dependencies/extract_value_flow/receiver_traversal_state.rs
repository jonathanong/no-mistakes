use super::*;

type Key = (usize, String);
type Paths = FxHashMap<(Key, ValueMutationPath), Vec<nested::ReceiverRead>>;
#[derive(Default)]
enum Visits {
    #[default]
    Empty,
    First {
        state: (Key, ValueMutationPath),
        reads: Vec<nested::ReceiverRead>,
    },
    Many(Paths),
}
impl Visits {
    fn enter(
        &mut self,
        key: &Key,
        path: &[Option<String>],
        reads: Vec<nested::ReceiverRead>,
    ) -> Option<Vec<nested::ReceiverRead>> {
        let state = (key.clone(), path.to_vec());
        match self {
            Self::Empty => {
                *self = Self::First {
                    state,
                    reads: reads.clone(),
                };
                Some(reads)
            }
            Self::First {
                state: previous_state,
                reads: previous,
            } if *previous_state == state => {
                merge_reads(previous, &reads).then(|| previous.clone())
            }
            Self::First {
                state: previous_state,
                reads: previous,
            } => {
                let previous_state = std::mem::take(previous_state);
                let previous = std::mem::take(previous);
                let mut entries = fx_map();
                entries.insert(previous_state, previous);
                entries.insert(state, reads.clone());
                *self = Self::Many(entries);
                Some(reads)
            }
            Self::Many(entries) => {
                let Some(previous) = entries.get_mut(&state) else {
                    entries.insert(state, reads.clone());
                    return Some(reads);
                };
                merge_reads(previous, &reads).then(|| previous.clone())
            }
        }
    }
}

/// Cycle membership uses the binding key, independently of visited paths.
#[derive(Default)]
pub(super) struct ActiveKeys {
    first: Option<Key>,
    other: FxHashSet<Key>,
}
impl ActiveKeys {
    pub(super) fn contains(&self, key: &Key) -> bool {
        self.first.as_ref() == Some(key) || self.other.contains(key)
    }
    fn insert(&mut self, key: Key) {
        if self.first.is_none() {
            self.first = Some(key);
        } else {
            self.other.insert(key);
        }
    }
    pub(super) fn remove(&mut self, key: &Key) -> bool {
        if self.first.as_ref() == Some(key) {
            self.first = None;
            true
        } else {
            self.other.remove(key)
        }
    }
}
#[derive(Default)]
pub(super) struct TraversalState {
    pub(super) active: ActiveKeys,
    visited: Visits,
}
impl TraversalState {
    pub(super) fn enter(
        &mut self,
        key: &Key,
        path: &[Option<String>],
        reads: Vec<nested::ReceiverRead>,
    ) -> Option<Vec<nested::ReceiverRead>> {
        if self.active.contains(key) {
            return None;
        }
        let reads = self.visited.enter(key, path, reads)?;
        self.active.insert(key.clone());
        Some(reads)
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn heap_buffers(&self) -> usize {
        usize::from(matches!(self.visited, Visits::Many(_)))
            + usize::from(self.active.other.capacity() > 0)
    }
}
/// One monotone cutoff summary per receiver/path. Keep the original zip,
/// minimum-offset, ambiguous-context and unchanged-arrival semantics.
fn merge_reads(previous: &mut [nested::ReceiverRead], reads: &[nested::ReceiverRead]) -> bool {
    let mut changed = false;
    for (previous, incoming) in previous.iter_mut().zip(reads) {
        let merged = match (*previous, *incoming) {
            (Some((left, context)), Some((right, other))) if context == other => {
                Some((left.min(right), context))
            }
            _ => None,
        };
        changed |= *previous != merged;
        *previous = merged;
    }
    changed
}
