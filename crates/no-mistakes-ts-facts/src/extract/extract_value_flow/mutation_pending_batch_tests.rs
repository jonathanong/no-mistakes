use super::*;
use std::{cell::RefCell, rc::Rc};

#[test]
fn batch_append_matches_vec_for_every_marker_position_and_known_output_shape() {
    for length in [0, 1, 2, 4] {
        for index in 0..=length {
            for first in [None, Some(10)] {
                for tail_length in [0, 1, 2, 32] {
                    for heap in [false, true] {
                        let mut initial = Vec::with_capacity(80);
                        initial.extend(0..length);
                        let mut queue = if heap {
                            Queue::from_vec(initial.clone())
                        } else {
                            let mut q = Queue::new(0);
                            q.pop();
                            q.extend(initial.clone());
                            q
                        };
                        let mut oracle = queue.into_vec();
                        queue = if heap {
                            Queue::from_vec(oracle.clone())
                        } else {
                            let mut q = Queue::new(0);
                            q.pop();
                            q.extend(oracle.clone());
                            q
                        };
                        let tail: Vec<_> = (0..tail_length).map(|v| 20 + v).collect();
                        oracle.reserve(tail.len() + usize::from(first.is_some()) + 1);
                        oracle.extend(first.into_iter().chain(tail.clone()).map(|v| v * 2));
                        oracle.insert(index, 99);
                        queue.append_with_exit(index, (first, tail), |v| v * 2, 99);
                        assert_eq!(queue.len(), oracle.len());
                        if let Queue::Heap(values) = &queue {
                            assert_eq!(values.capacity(), oracle.capacity());
                        }
                        while let Some(expected) = oracle.pop() {
                            assert_eq!(queue.pop(), Some(expected));
                        }
                        assert_eq!(queue.pop(), None);
                    }
                }
            }
        }
    }
}

#[test]
fn batch_inline_empty_and_single_outputs_keep_exact_exit_order_without_heap() {
    for first in [None, Some("successor".to_owned())] {
        let mut queue = Queue::new("old".to_owned());
        queue.pop();
        queue.append_with_exit(0, (first.clone(), Vec::new()), |v| v, "exit".into());
        assert_eq!(queue.capacity(), 0);
        if let Some(first) = first {
            assert_eq!(queue.pop(), Some(first));
        }
        assert_eq!(queue.pop().as_deref(), Some("exit"));
        assert_eq!(queue.pop(), None);
    }
}

#[test]
fn batch_from_vec_keeps_existing_buffer_and_virtual_headroom() {
    let mut initial = Vec::with_capacity(20);
    initial.extend([1, 2]);
    let pointer = initial.as_ptr();
    let mut queue = Queue::from_vec(initial);
    queue.append_with_exit(1, (Some(3), vec![4, 5]), |v| v, 99);
    match &queue {
        Queue::Heap(values) => {
            assert_eq!(values.as_ptr(), pointer);
            assert_eq!(values.capacity(), 20);
            assert_eq!(values, &[1, 99, 2, 3, 4, 5]);
        }
        Queue::Inline { .. } => panic!("batch must retain original Vec storage"),
    }
}

#[test]
fn batch_invalid_index_and_zst_length_overflow_leave_queue_intact() {
    let mut queue = Queue::new(());
    let invalid = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        queue.append_with_exit(2, (None::<()>, Vec::new()), |v| v, ());
    }));
    assert!(invalid.is_err());
    assert_eq!(queue.len(), 1);
    let overflow = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        queue.append_with_exit(0, (Some(()), vec![(); usize::MAX]), |v| v, ());
    }));
    assert!(overflow.is_err());
    assert_eq!(queue.len(), 1);
    // Exercise overflow in reserve's existing length + additional check too.
    let overflow = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        queue.append_with_exit(0, (None, vec![(); usize::MAX - 1]), |v| v, ());
    }));
    assert!(overflow.is_err());
    assert_eq!(queue.pop(), Some(()));
}

#[derive(Debug)]
struct Token {
    id: u32,
    dropped: Rc<RefCell<Vec<u32>>>,
}
impl Drop for Token {
    fn drop(&mut self) {
        self.dropped.borrow_mut().push(self.id);
    }
}

#[test]
fn batch_mapper_panic_drops_each_owned_value_once_and_keeps_inserted_prefix() {
    for tail_count in [0, 2] {
        let dropped = Rc::new(RefCell::new(Vec::new()));
        let token = |id| Token {
            id,
            dropped: Rc::clone(&dropped),
        };
        let mut queue = Queue::new(token(0));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            queue.append_with_exit(
                0,
                (Some(token(1)), (2..2 + tail_count).map(token).collect()),
                |value| {
                    if value.id == 1 + u32::from(tail_count > 0) {
                        panic!("saved mapper failure");
                    }
                    value
                },
                token(9),
            );
        }));
        assert!(result.is_err());
        assert_eq!(queue.len(), 1 + usize::from(tail_count > 0));
        drop(queue);
        let mut actual = dropped.borrow().clone();
        actual.sort_unstable();
        let mut expected = vec![0, 1, 9];
        expected.extend(2..2 + tail_count);
        expected.sort_unstable();
        assert_eq!(actual, expected);
    }
}

#[test]
fn batch_mapper_panic_while_inline_drops_each_owned_value_once_and_keeps_prefix() {
    // Inline slots hold two values. A resident token plus the first output and
    // the exit marker promotes, which the heap case above already covers.
    let dropped = Rc::new(RefCell::new(Vec::new()));
    let token = |id| Token {
        id,
        dropped: Rc::clone(&dropped),
    };
    let mut queue = Queue::new(token(0));
    drop(queue.pop());
    assert!(
        matches!(&queue, Queue::Inline { len: 0, .. }) && queue.capacity() == 0,
        "storage must remain inline before the mapper panic",
    );
    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        queue.append_with_exit(
            0,
            (Some(token(1)), Vec::new()),
            |_| panic!("saved mapper failure"),
            token(9),
        );
    }));
    let panic = panicked.expect_err("mapper must panic");
    assert_eq!(
        panic.downcast_ref::<&str>().copied(),
        Some("saved mapper failure")
    );
    assert_eq!(queue.len(), 0, "retained prefix length");
    assert!(matches!(&queue, Queue::Inline { slots, len: 0 } if slots.iter().all(Option::is_none)));
    drop(queue);
    let mut actual = dropped.borrow().clone();
    actual.sort_unstable();
    assert_eq!(actual, vec![0, 1, 9]);
}

impl<T> Queue<T> {
    pub(in super::super) fn insert(&mut self, index: usize, value: T) {
        #[cfg(all(test, feature = "test-instrumentation"))]
        work_counts::record(3);
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
        #[cfg(all(test, feature = "test-instrumentation"))]
        work_counts::record(2);
        let values = values.into_iter();
        self.reserve(values.size_hint().0);
        for value in values {
            self.push(value);
        }
    }
}
