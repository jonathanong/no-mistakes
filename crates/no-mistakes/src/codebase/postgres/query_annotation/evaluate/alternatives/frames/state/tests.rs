use super::*;
use crate::codebase::postgres::query_annotation::evaluate::ArgumentObjects;

#[test]
fn pruning_shares_one_walk_for_heap_mutations_and_builder_updates() {
    let scopes = vec![FxHashMap::from_iter([("args".into(), Value::Arguments(8))]).into()];
    let mut objects = ArgumentObjects::from_iter([
        (8, vec![Value::Prefix("SELECT 1".into(), true, Some(7))]),
        (9, vec![Value::Unknown]),
    ]);
    let mut extras = FxHashMap::from_iter([
        (
            8,
            std::collections::BTreeMap::from_iter([(
                2,
                Value::Prefix("tail".into(), true, Some(10)),
            )]),
        ),
        (9, Default::default()),
    ]);
    let mut deleted = FxHashSet::from_iter([(8, None), (9, Some(0))]);
    let mut definite = FxHashSet::from_iter([(8, 0), (9, 0)]);
    let mut disconnected = definite.clone();
    let mut invalidated = FxHashSet::from_iter([7, 9, 10, 99]);
    let mut updates = FxHashMap::from_iter([7, 9, 10, 99].map(|id| (id, Value::Unknown)));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    prune_state(
        &scopes,
        &[],
        MutationState {
            deleted: &mut deleted,
            definite: &mut definite,
            disconnected: &mut disconnected,
            invalidated: &mut invalidated,
            updates: &mut updates,
        },
        &Default::default(),
        ArenaMut {
            objects: &mut objects,
            extras: &mut extras,
        },
        &Default::default(),
        &[],
    );
    assert_eq!(
        observer.snapshot().work["postgres.annotation.reachability_walks"],
        1
    );
    assert_eq!(
        objects.keys().copied().collect::<FxHashSet<_>>(),
        FxHashSet::from_iter([8])
    );
    assert_eq!(
        extras.keys().copied().collect::<FxHashSet<_>>(),
        FxHashSet::from_iter([8])
    );
    assert_eq!(deleted, FxHashSet::from_iter([(8, None)]));
    assert_eq!(definite, FxHashSet::from_iter([(8, 0)]));
    assert_eq!(disconnected, FxHashSet::from_iter([(8, 0)]));
    assert_eq!(invalidated, FxHashSet::from_iter([7, 10]));
    assert_eq!(
        updates.keys().copied().collect::<FxHashSet<_>>(),
        FxHashSet::from_iter([7, 10])
    );
}

#[test]
fn pruning_skips_only_completely_empty_state() {
    // Each independent state table must force a walk, even when the others
    // are empty. A builder update alone still needs liveness filtering.
    for populated in 0..=7 {
        let mut objects = ArgumentObjects::default();
        let mut extras = FxHashMap::default();
        let mut deleted = FxHashSet::default();
        let mut definite = FxHashSet::default();
        let mut disconnected = FxHashSet::default();
        let mut invalidated = FxHashSet::default();
        let mut updates = FxHashMap::default();
        match populated {
            0 => {
                objects.insert(9, vec![]);
            }
            1 => {
                extras.insert(9, Default::default());
            }
            2 => {
                deleted.insert((9, None));
            }
            3 => {
                definite.insert((9, 0));
            }
            4 => {
                disconnected.insert((9, 0));
            }
            5 => {
                invalidated.insert(9);
            }
            6 => {
                updates.insert(9, Value::Unknown);
            }
            _ => {}
        }
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        prune_state(
            &[],
            &[Value::Prefix("SELECT 1".into(), true, None)],
            MutationState {
                deleted: &mut deleted,
                definite: &mut definite,
                disconnected: &mut disconnected,
                invalidated: &mut invalidated,
                updates: &mut updates,
            },
            &Default::default(),
            ArenaMut {
                objects: &mut objects,
                extras: &mut extras,
            },
            &Default::default(),
            &[],
        );
        let walks = observer
            .snapshot()
            .work
            .get("postgres.annotation.reachability_walks")
            .copied()
            .unwrap_or(0);
        assert_eq!(walks, u64::from(populated < 7));
        assert!(
            objects.is_empty()
                && extras.is_empty()
                && deleted.is_empty()
                && definite.is_empty()
                && disconnected.is_empty()
                && invalidated.is_empty()
                && updates.is_empty()
        );
    }
}
