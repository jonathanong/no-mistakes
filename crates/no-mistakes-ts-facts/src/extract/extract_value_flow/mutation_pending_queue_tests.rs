use super::*;
use std::cell::Cell;
use std::rc::Rc;

fn contents(queue: &Queue<usize>) -> Vec<usize> {
    match queue {
        Queue::Inline { slots, len } => {
            assert!(*len <= 2);
            assert!(slots[*len..].iter().all(Option::is_none));
            assert_eq!(queue.capacity(), 0, "inline storage is not a heap buffer");
            slots[..*len].iter().map(|value| value.unwrap()).collect()
        }
        Queue::Heap(values) => {
            assert!(queue.capacity() >= values.len());
            values.clone()
        }
    }
}

#[test]
fn empty_one_and_two_entries_keep_heap_storage_cold() {
    let mut queue = Queue::new(10);
    assert_eq!(contents(&queue), [10]);
    assert_eq!(queue.pop(), Some(10));
    assert_eq!(queue.pop(), None);
    queue.reserve(2);
    assert!(contents(&queue).is_empty());
    queue.insert(0, 20);
    queue.push(30);
    assert_eq!(contents(&queue), [20, 30]);
    assert_eq!(queue.capacity(), 0);
    assert_eq!(queue.pop(), Some(30));
    assert_eq!(queue.pop(), Some(20));
    assert_eq!(queue.pop(), None);
    queue.push(40);
    assert_eq!(contents(&queue), [40]);
    assert_eq!(queue.capacity(), 0);
}

#[test]
fn exit_markers_preserve_vec_lifo_through_wide_and_repeated_branches() {
    for fanout in [0, 1, 2, 4, 32] {
        let mut queue = Queue::new(0);
        let mut oracle = vec![0];
        assert_eq!(queue.pop(), oracle.pop());
        // Mirror the consumer's ordering, including its virtual exit headroom.
        let exit_position = queue.len();
        queue.reserve(fanout + 1);
        oracle.reserve(fanout + 1);
        for child in 1..=fanout {
            queue.push(child);
            oracle.push(child);
        }
        queue.insert(exit_position, 100);
        oracle.insert(exit_position, 100);
        assert_eq!(contents(&queue), oracle);
        if fanout > 0 {
            assert_eq!(queue.pop(), oracle.pop());
            let nested_exit = queue.len();
            // Repeated identities must remain queued; the queue imposes no cutoff.
            queue.extend([7, 7, 8, 7]);
            oracle.extend([7, 7, 8, 7]);
            queue.insert(nested_exit, 200);
            oracle.insert(nested_exit, 200);
        }
        while !oracle.is_empty() {
            assert_eq!(contents(&queue), oracle);
            assert_eq!(queue.pop(), oracle.pop());
            assert_eq!(queue.len(), oracle.len());
        }
        assert_eq!(queue.pop(), None);
    }
}

#[test]
fn reserve_promotes_only_when_requested_headroom_exceeds_inline_storage() {
    let mut queue = Queue::new(0);
    queue.reserve(1);
    assert_eq!(queue.capacity(), 0);
    queue.reserve(20);
    assert!(queue.capacity() >= 21);
    let reserved = queue.capacity();
    queue.extend(1..=20);
    assert_eq!(contents(&queue), (0..=20).collect::<Vec<_>>());
    assert_eq!(
        queue.capacity(),
        reserved,
        "the reserved batch must not grow again"
    );
    queue.insert(10, 99);
    let mut oracle = (0..=20).collect::<Vec<_>>();
    oracle.insert(10, 99);
    assert_eq!(contents(&queue), oracle);
}

#[test]
fn invalid_insert_and_capacity_overflow_leave_existing_values_intact() {
    let mut queue = Queue::new(1);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        queue.insert(2, 99);
    }))
    .is_err());
    assert_eq!(contents(&queue), [1]);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        queue.reserve(usize::MAX);
    }))
    .is_err());
    assert_eq!(contents(&queue), [1]);
    queue.extend(2..=4);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        queue.insert(5, 99);
    }))
    .is_err());
    assert_eq!(contents(&queue), [1, 2, 3, 4]);
}

#[test]
fn noncopy_values_and_failed_insert_are_dropped_exactly_once() {
    struct Tracked {
        name: String,
        id: usize,
        drops: Rc<[Cell<usize>; 4]>,
    }
    impl Drop for Tracked {
        fn drop(&mut self) {
            self.drops[self.id].set(self.drops[self.id].get() + 1);
        }
    }
    let drops = Rc::new(std::array::from_fn(|_| Cell::new(0)));
    let value = |id| Tracked {
        name: format!("receiver-{id}"),
        id,
        drops: Rc::clone(&drops),
    };
    {
        let mut queue = Queue::new(value(0));
        queue.push(value(1));
        queue.insert(1, value(2));
        queue.reserve(20);
        assert!(drops.iter().all(|count| count.get() == 0));
        let popped = queue.pop().unwrap();
        assert_eq!(popped.name, "receiver-1");
        drop(popped);
        assert_eq!(drops[1].get(), 1);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            queue.insert(99, value(3));
        }))
        .is_err());
        assert_eq!(drops[3].get(), 1);
        assert_eq!(queue.pop().unwrap().name, "receiver-2");
    }
    assert!(drops.iter().all(|count| count.get() == 1));
}

impl<T> Queue<T> {
    pub(crate) fn into_vec(self) -> Vec<T> {
        match self {
            Self::Inline { slots, len } => {
                let mut values = Vec::with_capacity(len);
                values.extend(slots.into_iter().take(len).map(Option::unwrap));
                values
            }
            Self::Heap(values) => values,
        }
    }
}

#[test]
fn existing_vec_batches_keep_their_buffer_capacity_and_complete_lifo_order() {
    for length in [0, 1, 2, 6, 32] {
        let mut values = Vec::with_capacity(40);
        values.extend((0..length).map(|index| format!("receiver-{index}")));
        let pointer = values.as_ptr();
        let capacity = values.capacity();
        let mut oracle = values.clone();
        let mut queue = Queue::from_vec(values);
        assert_eq!(queue.len(), length);
        assert_eq!(queue.capacity(), capacity);
        match &queue {
            Queue::Heap(values) => assert_eq!(values.as_ptr(), pointer),
            Queue::Inline { .. } => panic!("existing Vec storage must pass through unchanged"),
        }
        // A replay batch shares one queue; duplicates and empty batches are not cut off.
        while let Some(expected) = oracle.pop() {
            assert_eq!(queue.pop(), Some(expected));
        }
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.capacity(), capacity);
    }
}

#[test]
fn unallocated_empty_vec_and_singleton_batches_accept_exit_insertion() {
    for values in [Vec::new(), vec![7]] {
        let mut oracle = values.clone();
        let capacity = values.capacity();
        let mut queue = Queue::from_vec(values);
        assert_eq!(queue.capacity(), capacity);
        let exit_position = queue.len();
        queue.extend([9, 9, 10]);
        oracle.extend([9, 9, 10]);
        queue.insert(exit_position, 99);
        oracle.insert(exit_position, 99);
        while !oracle.is_empty() {
            assert_eq!(contents(&queue), oracle);
            assert_eq!(queue.pop(), oracle.pop());
        }
        assert_eq!(queue.pop(), None);
    }
}
