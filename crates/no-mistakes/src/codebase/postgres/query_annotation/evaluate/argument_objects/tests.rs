use super::super::Value;
use super::*;
use crate::diagnostics::{with_observer, InvocationObserver};

fn copied(observer: &InvocationObserver) -> u64 {
    observer
        .snapshot()
        .work
        .get("postgres.annotation.dense_arena_slots_copied")
        .copied()
        .unwrap_or_default()
}

#[test]
fn arena_snapshots_copy_only_the_written_vector_and_keep_siblings_unchanged() {
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        let mut parent = ArgumentObjects::default();
        parent.insert(
            7,
            vec![Value::Arguments(7), Value::Primitive, Value::Unknown],
        );
        parent.extend(ArgumentObjects::from_iter([(
            8,
            vec![Value::Arguments(8); 64],
        )]));
        let sibling = parent.clone();
        let mut changed = parent.clone();
        assert_eq!(copied(&observer), 0);
        changed.get_mut(&7).unwrap()[1] = Value::Arguments(8);
        assert_eq!(
            copied(&observer),
            3,
            "only the written three-slot vector detaches"
        );
        assert!(parent[&7][1] == Value::Primitive);
        assert!(sibling[&7][1] == Value::Primitive);
        assert!(changed[&7][1] == Value::Arguments(8));
        assert_ne!(changed[&7].as_ptr(), parent[&7].as_ptr());
        assert_eq!(changed[&8].as_ptr(), parent[&8].as_ptr());
        assert_eq!(sibling[&7].as_ptr(), parent[&7].as_ptr());
        changed.remove(&8);
        assert!(parent.contains_key(&8) && sibling.contains_key(&8));
        assert_eq!(copied(&observer), 3);
    });
}

#[test]
fn conversions_extend_and_owned_iteration_keep_dense_snapshots_shared() {
    let parent = ArgumentObjects::from(FxHashMap::from_iter([(7, vec![Value::Arguments(7)])]));
    let mut child = ArgumentObjects::default();
    child.extend(parent.clone());
    let replaced = child.insert(7, parent[&7].clone()).unwrap();
    assert_eq!(replaced.as_ptr(), parent[&7].as_ptr());
    let collected = (&child)
        .into_iter()
        .map(|(id, slots)| (*id, slots.clone()))
        .collect::<ArgumentObjects>();
    assert!(collected == parent);
    for (_, slots) in collected {
        assert_eq!(slots.as_ptr(), parent[&7].as_ptr());
    }
    for (_, slots) in &mut child {
        assert_eq!(slots.len(), 1);
    }
}

#[test]
fn observer_off_and_timings_only_leave_copy_counters_empty() {
    with_observer(None, || {
        let parent = ArgumentObjects::from_iter([(7, vec![Value::Arguments(7)])]);
        let mut changed = parent.clone();
        changed.get_mut(&7).unwrap()[0] = Value::Primitive;
        assert!(crate::diagnostics::current().is_none());
        assert!(parent[&7][0] == Value::Arguments(7));
    });
    let observer = InvocationObserver::new(false);
    with_observer(Some(observer.clone()), || {
        let parent = ArgumentObjects::from_iter([(7, vec![Value::Arguments(7)])]);
        let mut changed = parent.clone();
        changed.get_mut(&7).unwrap().clear();
        assert_eq!(parent[&7].len(), 1);
    });
    assert!(observer.snapshot().work.is_empty());
}
