use super::*;
use crate::codebase::postgres::query_annotation::evaluate::Value;
use crate::fx::{FxHashMap, FxHashSet};

#[test]
fn snapshot_pruning_reuses_compaction_for_cyclic_arguments_and_builder_updates() {
    let path = PathBuf::from("module.mts");
    let files = FxHashMap::default();
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |_: &str, _: &Path| -> Option<PathBuf> { None },
        events: Default::default(),
        // The unrooted frame deliberately precedes the module, forcing a remap.
        scopes: vec![
            FxHashMap::from_iter([("dead".into(), Value::Prefix("dead".into(), true, Some(9)))])
                .into(),
            FxHashMap::from_iter([
                ("live".into(), Value::Prefix("live".into(), true, Some(7))),
                ("arguments".into(), Value::Arguments(42)),
            ])
            .into(),
            FxHashMap::from_iter([("arguments".into(), Value::Arguments(43))]).into(),
        ],
        modules: FxHashMap::from_iter([(path.clone(), 1)]),
        active_module_initials: Default::default(),
        transient_roots: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        untrusted_array_intrinsics: false,
        next_builder: 10,
        invalidated_builders: FxHashSet::from_iter([7, 9, 11, 12, 42, 43]),
        builder_updates: FxHashMap::from_iter(
            [7, 9, 11, 12, 42, 43].map(|id| (id, Value::Unknown)),
        ),
        // Mutually captured frames and argument cycles must remain finite.
        captured_bindings: FxHashMap::from_iter([
            (1, FxHashMap::from_iter([("owner".into(), 2)])),
            (2, FxHashMap::from_iter([("owner".into(), 1)])),
        ]),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        mapped_parameter_indices: Default::default(),
        deleted_argument_slots: FxHashSet::from_iter([(42, None), (99, Some(0))]),
        argument_objects: FxHashMap::from_iter([
            (42, vec![Value::Arguments(43)]),
            (
                43,
                vec![
                    Value::Arguments(42),
                    Value::Prefix("tail".into(), true, Some(11)),
                ],
            ),
            (99, vec![Value::Prefix("dead".into(), true, Some(9))]),
        ]),
        argument_extra_slots: FxHashMap::from_iter([
            (
                43,
                std::collections::BTreeMap::from_iter([(
                    1000,
                    Value::Prefix("extra".into(), true, Some(12)),
                )]),
            ),
            (99, Default::default()),
        ]),
        definite_deleted_argument_slots: FxHashSet::from_iter([(43, 1), (99, 0)]),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: FxHashSet::from_iter([(43, 1), (99, 0)]),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        mapped_arguments: Default::default(),
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    evaluator.prune_snapshot_state();
    assert_eq!(evaluator.scopes.len(), 2);
    assert_eq!(evaluator.modules[&path], 0);
    assert!(matches!(
        evaluator.scopes[0]["live"],
        Value::Prefix(_, _, Some(7))
    ));
    assert_eq!(
        evaluator.invalidated_builders,
        FxHashSet::from_iter([7, 11, 12, 42, 43])
    );
    assert_eq!(
        evaluator
            .builder_updates
            .keys()
            .copied()
            .collect::<FxHashSet<_>>(),
        FxHashSet::from_iter([7, 11, 12, 42, 43])
    );
    assert_eq!(
        evaluator
            .argument_objects
            .keys()
            .copied()
            .collect::<FxHashSet<_>>(),
        FxHashSet::from_iter([42, 43])
    );
    assert!(evaluator.argument_objects[&42][0] == Value::Arguments(43));
    assert!(evaluator.argument_objects[&43][0] == Value::Arguments(42));
    assert_eq!(evaluator.argument_extra_slots.len(), 1);
    assert_eq!(
        evaluator.deleted_argument_slots,
        FxHashSet::from_iter([(42, None)])
    );
    assert_eq!(
        evaluator.definite_deleted_argument_slots,
        FxHashSet::from_iter([(43, 1)])
    );
    assert_eq!(
        evaluator.disconnected_argument_slots,
        FxHashSet::from_iter([(43, 1)])
    );
    assert_eq!(evaluator.captured_bindings[&0]["owner"], 1);
    assert_eq!(evaluator.captured_bindings[&1]["owner"], 0);
    let mut targets = evaluator.captured_write_targets(0, "owner");
    targets.sort_unstable();
    assert_eq!(targets, vec![0, 1]);
    // Compaction already found exactly the module-rooted mutation identities.
    assert_eq!(
        observer.snapshot().work["postgres.annotation.reachability_walks"],
        1
    );
}
