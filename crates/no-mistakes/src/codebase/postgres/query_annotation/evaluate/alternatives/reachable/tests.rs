use super::*;

#[test]
fn shared_containers_are_visited_once_per_reachability_collection() {
    let mut value = Value::Possible(
        vec![
            Value::Prefix("SELECT 1".into(), true, Some(7)),
            Value::Arguments(8),
        ]
        .into(),
    );
    // A shared DAG has exponentially many paths but only thirteen containers.
    for _ in 0..12 {
        value = Value::Aggregate(vec![value.clone(), value].into());
    }
    let mut reachable = Reachable::default();
    reachable.value(&value);
    reachable.value(&value);
    assert_eq!(reachable.visited_values.len(), 13);
    assert_eq!(reachable.identities, FxHashSet::from_iter([7]));
    assert_eq!(reachable.pending_args, vec![8]);
}
