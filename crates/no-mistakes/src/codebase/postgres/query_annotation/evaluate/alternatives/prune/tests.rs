use super::*;
use crate::codebase::postgres::query_annotation::evaluate::Value;
use crate::fx::{FxHashMap, FxHashSet};

#[test]
fn snapshot_pruning_retains_only_module_reachable_builder_updates() {
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
            FxHashMap::from_iter([("live".into(), Value::Prefix("live".into(), true, Some(7)))])
                .into(),
        ],
        modules: FxHashMap::from_iter([(path.clone(), 1)]),
        active_module_initials: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        next_builder: 10,
        invalidated_builders: FxHashSet::from_iter([7, 9]),
        builder_updates: FxHashMap::from_iter([(7, Value::Unknown), (9, Value::Unknown)]),
        captured_bindings: Default::default(),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        mapped_parameter_indices: Default::default(),
        deleted_argument_slots: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        mapped_arguments: Default::default(),
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    evaluator.prune_snapshot_state();
    assert_eq!(evaluator.scopes.len(), 1);
    assert_eq!(evaluator.modules[&path], 0);
    assert!(matches!(
        evaluator.scopes[0]["live"],
        Value::Prefix(_, _, Some(7))
    ));
    assert_eq!(evaluator.invalidated_builders, FxHashSet::from_iter([7]));
    assert_eq!(
        evaluator
            .builder_updates
            .keys()
            .copied()
            .collect::<FxHashSet<_>>(),
        FxHashSet::from_iter([7])
    );
    // One compaction walk plus one shared mutation/update pruning walk.
    assert_eq!(
        observer.snapshot().work["postgres.annotation.reachability_walks"],
        2
    );
}
