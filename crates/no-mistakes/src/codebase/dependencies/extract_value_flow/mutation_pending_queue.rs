#[cfg(feature = "test-instrumentation")]
use super::pending_work::record_queue;

pub(in super::super) type PendingMutation<R = super::nested::RootRead> =
    (super::nested::ReceiverWrite<R>, bool, bool, bool);

pub(in super::super) enum Queue<T> {
    Inline { slots: [Option<T>; 2], len: usize },
    Heap(Vec<T>),
}

impl<T> Queue<T> {
    pub(in super::super) fn new(initial: T) -> Self {
        #[cfg(all(test, feature = "test-instrumentation"))]
        work_counts::record(7);
        Self::Inline {
            slots: [Some(initial), None],
            len: 1,
        }
    }

    pub(in super::super) fn from_vec(values: Vec<T>) -> Self {
        #[cfg(feature = "test-instrumentation")]
        record_queue(values.len(), 0);
        Self::Heap(values)
    }

    pub(in super::super) fn len(&self) -> usize {
        match self {
            Self::Inline { len, .. } => *len,
            Self::Heap(values) => values.len(),
        }
    }

    #[cfg(any(test, feature = "test-instrumentation"))]
    pub(in super::super) fn capacity(&self) -> usize {
        match self {
            Self::Inline { .. } => 0,
            Self::Heap(values) => values.capacity(),
        }
    }

    pub(in super::super) fn reserve(&mut self, additional: usize) {
        #[cfg(all(test, feature = "test-instrumentation"))]
        work_counts::record(0);
        let required = self
            .len()
            .checked_add(additional)
            .expect("capacity overflow");
        match self {
            Self::Inline { slots, len } if required > 2 => {
                #[cfg(all(test, feature = "test-instrumentation"))]
                work_counts::record(5);
                let mut values = Vec::with_capacity(4.max(required));
                values.extend(slots.iter_mut().take(*len).map(|slot| slot.take().unwrap()));
                *self = Self::Heap(values);
                #[cfg(feature = "test-instrumentation")]
                record_queue(self.len(), 1);
            }
            Self::Heap(values) => {
                #[cfg(feature = "test-instrumentation")]
                let capacity = values.capacity();
                values.reserve(additional);
                #[cfg(feature = "test-instrumentation")]
                record_queue(values.len(), usize::from(values.capacity() != capacity));
            }
            Self::Inline { .. } => {}
        }
    }

    pub(in super::super) fn push(&mut self, value: T) {
        #[cfg(all(test, feature = "test-instrumentation"))]
        work_counts::record(1);
        self.reserve(1);
        match self {
            Self::Inline { slots, len } => {
                slots[*len] = Some(value);
                *len += 1;
            }
            Self::Heap(values) => values.push(value),
        }
    }

    /// Append known output parts and place the exit below their LIFO successors.
    /// The exact owned lengths retain the old virtual exit capacity headroom.
    pub(in super::super) fn append_with_exit<U>(
        &mut self,
        index: usize,
        (first, tail): (Option<U>, Vec<U>),
        mut map: impl FnMut(U) -> T,
        exit: T,
    ) {
        assert!(index <= self.len());
        let additional = tail
            .len()
            .checked_add(usize::from(first.is_some()))
            .and_then(|count| count.checked_add(1))
            .expect("capacity overflow");
        self.reserve(additional);
        match self {
            Self::Inline { slots, len } => {
                for value in first.into_iter().chain(tail) {
                    slots[*len] = Some(map(value));
                    *len += 1;
                }
                for slot in (index..*len).rev() {
                    slots[slot + 1] = slots[slot].take();
                }
                slots[index] = Some(exit);
                *len += 1;
            }
            Self::Heap(values) => {
                values.extend(first.into_iter().chain(tail).map(map));
                values.insert(index, exit);
            }
        }
    }

    pub(in super::super) fn pop(&mut self) -> Option<T> {
        #[cfg(all(test, feature = "test-instrumentation"))]
        work_counts::record(6);
        match self {
            Self::Inline { slots, len } => {
                if *len == 0 {
                    return None;
                }
                *len -= 1;
                slots[*len].take()
            }
            Self::Heap(values) => values.pop(),
        }
    }
}

#[cfg(test)]
#[path = "mutation_pending_queue_tests.rs"]
mod tests;

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "mutation_queue_work_counts_tests.rs"]
pub(super) mod work_counts;

#[cfg(test)]
#[path = "mutation_pending_batch_tests.rs"]
mod batch_tests;
