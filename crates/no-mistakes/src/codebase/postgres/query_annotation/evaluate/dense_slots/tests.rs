use super::*;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::diagnostics::{with_observer, InvocationObserver};
use crate::fx::FxHashSet;

fn callback(env: usize) -> Value {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/postgres-branch-copies/src/dense.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let mut facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@bench/db", &[]),
        )
    })
    .unwrap();
    let query_annotation::Expr::Function(function) = facts.globals.remove("densePayload").unwrap()
    else {
        panic!("saved dense helper function");
    };
    Value::Function(function, path.into(), env)
}

fn copied(observer: &InvocationObserver) -> u64 {
    observer
        .snapshot()
        .work
        .get("postgres.annotation.dense_arena_slots_copied")
        .copied()
        .unwrap_or_default()
}

#[test]
fn readonly_snapshots_share_dense_storage_and_warmed_edges_without_copies() {
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        let slots = DenseSlots::from(vec![Value::Arguments(7), Value::Primitive]);
        assert!(!slots.shared());
        let edges = slots.references() as *const Edges;
        let mut snapshots = vec![slots.clone(); 16];
        assert!(slots.shared());
        snapshots[0].clone_from(&slots);
        for snapshot in snapshots {
            assert!(snapshot == slots);
            assert!(std::ptr::eq(snapshot.as_ptr(), slots.as_ptr()));
            assert_eq!(snapshot.references() as *const Edges, edges);
            assert_eq!((&snapshot).into_iter().count(), 2);
        }
        assert_eq!(copied(&observer), 0);
        assert!(slots == DenseSlots::from(vec![Value::Arguments(7), Value::Primitive]));
        assert!(slots != DenseSlots::from(vec![Value::Primitive]));
    });
}

#[test]
fn detaching_invalidates_direct_edges_and_preserves_parent_cycles() {
    let slots = DenseSlots::from(vec![
        callback(3),
        Value::Arguments(7),
        Value::Prefix("SELECT 1".into(), true, Some(9)),
    ]);
    assert_eq!(slots.references().environments, FxHashSet::from_iter([3]));
    assert_eq!(slots.references().arguments, FxHashSet::from_iter([7]));
    assert!(slots.references().identities.contains(&9));
    let mut changed = slots.clone();
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        for value in &mut changed {
            *value = Value::Arguments(11);
        }
        assert!(changed.0.references.get().is_none());
        assert_eq!(changed.references().arguments, FxHashSet::from_iter([11]));
        assert!(changed.references().environments.is_empty());
        assert!(!changed.references().identities.contains(&9));
        assert_eq!(copied(&observer), 3);
        changed.push(Value::Primitive);
        assert!(changed.0.references.get().is_none());
        assert_eq!(copied(&observer), 3, "unique writes do not copy again");
    });
    assert_eq!(slots.references().environments, FxHashSet::from_iter([3]));
    assert_eq!(slots.references().arguments, FxHashSet::from_iter([7]));
    assert_eq!(slots.len(), 3);
}

#[test]
fn nested_values_detach_before_mutation_and_refresh_outer_edges() {
    let slots = DenseSlots::from(vec![Value::Aggregate(vec![Value::Arguments(7)].into())]);
    assert_eq!(slots.references().arguments, FxHashSet::from_iter([7]));
    let mut changed = slots.clone();
    let Value::Aggregate(values) = &mut changed[0] else {
        unreachable!()
    };
    values.push(callback(4));
    values.push(Value::Prefix("SELECT nested".into(), true, Some(12)));
    assert!(changed.0.references.get().is_none());
    assert_eq!(changed.references().environments, FxHashSet::from_iter([4]));
    assert!(changed.references().identities.contains(&12));
    assert!(slots.references().environments.is_empty());
    assert!(!slots.references().identities.contains(&12));
    let (Value::Aggregate(parent), Value::Aggregate(child)) = (&slots[0], &changed[0]) else {
        unreachable!()
    };
    assert_ne!(parent.identity(), child.identity());
    assert_eq!(parent.len(), 1);
}

#[test]
fn consuming_shared_storage_counts_real_copies_but_unique_storage_moves() {
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        let slots = DenseSlots::from(vec![Value::Arguments(7), Value::Primitive]);
        let snapshot = slots.clone();
        let moved = slots.into_iter().collect::<Vec<_>>();
        assert_eq!(copied(&observer), 2);
        assert_eq!(snapshot.len(), 2);
        let unique = DenseSlots::from(moved);
        let pointer = unique.as_ptr();
        let moved = unique.into_iter();
        assert_eq!(pointer, moved.as_slice().as_ptr());
        assert_eq!(copied(&observer), 2);
    });
}
