use super::*;

#[cfg(feature = "test-instrumentation")]
thread_local! {
    pub(super) static FIRST_ALLOCATIONS: std::cell::Cell<[usize; 3]> = const { std::cell::Cell::new([0; 3]) };
}
#[cfg(feature = "test-instrumentation")]
fn first_allocation(index: usize, allocated: bool) {
    FIRST_ALLOCATIONS.with(|counts| {
        let mut values = counts.get();
        values[index] += usize::from(allocated);
        counts.set(values);
    });
}

type Key = (usize, String);
type Paths = FxHashMap<(Key, ValueMutationPath), Vec<nested::ReceiverRead>>;

#[derive(Default)]
pub(super) enum TraversalState {
    #[default]
    Empty,
    FirstInline {
        state: (Key, Option<Option<String>>),
        read: Option<nested::ReceiverRead>,
        active: bool,
    },
    FirstHeap {
        state: (Key, ValueMutationPath),
        reads: Vec<nested::ReceiverRead>,
        active: bool,
    },
    Many {
        visited: Paths,
        active: ActiveKeys,
    },
}

#[path = "receiver_traversal_active.rs"]
mod active;
use active::ActiveKeys;

impl TraversalState {
    pub(super) fn contains_active(&self, key: &Key) -> bool {
        match self {
            Self::Empty => false,
            Self::FirstInline { state, active, .. } => *active && state.0 == *key,
            Self::FirstHeap { state, active, .. } => *active && state.0 == *key,
            Self::Many { active, .. } => active.contains(key),
        }
    }
    pub(super) fn remove_active(&mut self, key: &Key) -> bool {
        match self {
            Self::FirstInline { state, active, .. } if state.0 == *key => std::mem::take(active),
            Self::FirstHeap { state, active, .. } if state.0 == *key => std::mem::take(active),
            Self::Many { active, .. } => active.remove(key),
            _ => false,
        }
    }
    pub(super) fn enter(
        &mut self,
        key: &Key,
        path: &[Option<String>],
        reads: Vec<nested::ReceiverRead>,
    ) -> Option<Vec<nested::ReceiverRead>> {
        // Active identity is the binding alone, before path matching/promotion.
        if self.contains_active(key) {
            return None;
        }
        match self {
            Self::Empty if path.len() <= 1 && reads.len() <= 1 => {
                *self = Self::FirstInline {
                    state: (key.clone(), path.first().cloned()),
                    read: reads.first().copied(),
                    active: true,
                };
                Some(reads)
            }
            Self::Empty => {
                #[cfg(feature = "test-instrumentation")]
                {
                    first_allocation(0, !reads.is_empty());
                    first_allocation(1, !path.is_empty());
                }
                *self = Self::FirstHeap {
                    state: (key.clone(), path.to_vec()),
                    reads: reads.clone(),
                    active: true,
                };
                Some(reads)
            }
            Self::FirstInline {
                state,
                read,
                active,
            } if state.0 == *key && state.1.as_slice() == path => {
                let changed = merge_reads(read.as_mut_slice(), &reads);
                *active = changed;
                changed.then(|| read.as_slice().to_vec())
            }
            Self::FirstHeap {
                state,
                reads: previous,
                active,
            } if state.0 == *key && state.1 == path => {
                let changed = merge_reads(previous, &reads);
                *active = changed;
                changed.then(|| previous.clone())
            }
            Self::FirstInline { .. } | Self::FirstHeap { .. } => {
                self.promote();
                self.enter(key, path, reads)
            }
            Self::Many { visited, active } => {
                let state = (key.clone(), path.to_vec());
                let selected = if let Some(previous) = visited.get_mut(&state) {
                    merge_reads(previous, &reads).then(|| previous.clone())
                } else {
                    visited.insert(state, reads.clone());
                    Some(reads)
                }?;
                active.insert(key.clone());
                Some(selected)
            }
        }
    }
    fn promote(&mut self) {
        let (state, reads, was_active) = match std::mem::take(self) {
            Self::FirstInline {
                state,
                read,
                active,
            } => {
                #[cfg(feature = "test-instrumentation")]
                {
                    first_allocation(0, read.is_some());
                    first_allocation(1, state.1.is_some());
                }
                (
                    (state.0, state.1.into_iter().collect()),
                    read.into_iter().collect(),
                    active,
                )
            }
            Self::FirstHeap {
                state,
                reads,
                active,
            } => (state, reads, active),
            _ => unreachable!("only first visits promote"),
        };
        #[cfg(feature = "test-instrumentation")]
        first_allocation(2, was_active && !state.0 .1.is_empty());
        let mut active = ActiveKeys::default();
        // The owned visited key is about to move. Materialize an independent
        // active key first, only when its original traversal is still active.
        if was_active {
            active.insert(state.0.clone());
        }
        let mut visited = fx_map();
        visited.insert(state, reads);
        *self = Self::Many { visited, active };
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn heap_buffers(&self) -> usize {
        match self {
            Self::Many { active, .. } => 1 + active.overflow_buffer(),
            _ => 0,
        }
    }
}

/// Preserve zip cardinality, minimum offsets and ambiguous owner semantics.
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
