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
        let required = self
            .len()
            .checked_add(additional)
            .expect("capacity overflow");
        match self {
            Self::Inline { slots, len } if required > 2 => {
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
        self.reserve(1);
        match self {
            Self::Inline { slots, len } => {
                slots[*len] = Some(value);
                *len += 1;
            }
            Self::Heap(values) => values.push(value),
        }
    }

    pub(in super::super) fn pop(&mut self) -> Option<T> {
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

    pub(in super::super) fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len());
        self.reserve(1);
        match self {
            Self::Inline { slots, len } => {
                for slot in (index..*len).rev() {
                    slots[slot + 1] = slots[slot].take();
                }
                slots[index] = Some(value);
                *len += 1;
            }
            Self::Heap(values) => values.insert(index, value),
        }
    }

    pub(in super::super) fn extend(&mut self, values: impl IntoIterator<Item = T>) {
        let values = values.into_iter();
        self.reserve(values.size_hint().0);
        for value in values {
            self.push(value);
        }
    }
}

#[cfg(test)]
#[path = "mutation_pending_queue_tests.rs"]
mod tests;
