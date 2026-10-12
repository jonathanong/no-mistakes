use super::*;
use crate::codebase::postgres::query_annotation::evaluate::{File, Scope};
use crate::fx::FxHashSet;

fn evaluator<'a>(
    files: &'a FxHashMap<PathBuf, File<'a>>,
) -> Evaluator<'a, impl Fn(&str, &Path) -> Option<PathBuf>> {
    Evaluator {
        files,
        resolve: |_: &str, _: &Path| -> Option<PathBuf> { None },
        events: Default::default(),
        scopes: Vec::new(),
        modules: Default::default(),
        active_module_initials: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        next_builder: 0,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
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
    }
}

#[test]
fn equal_capture_refresh_keeps_shared_bindings_and_builder_indexes() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    let scope = Scope::from(FxHashMap::from_iter([
        (
            "builder".into(),
            Value::Prefix("SELECT 1".into(), true, Some(7)),
        ),
        ("unknown".into(), Value::Unknown),
    ]));
    assert!(scope.contains_builders(&FxHashSet::from_iter([7])));
    evaluator.scopes = vec![scope.clone(), scope.clone()];
    evaluator.captured_bindings.insert(
        1,
        FxHashMap::from_iter([("builder".into(), 0), ("unknown".into(), 0)]),
    );
    evaluator.refresh_captured_bindings();
    for frame in &evaluator.scopes {
        assert!(std::ptr::eq(&**frame, &*scope));
        assert!(frame.builder_index_initialized());
    }
    // Unknown is a binding value: absence must still become an own entry.
    evaluator
        .captured_bindings
        .get_mut(&1)
        .unwrap()
        .insert("missing".into(), 0);
    evaluator.refresh_captured_bindings();
    assert!(evaluator.scopes[1].get("missing") == Some(&Value::Unknown));
    assert!(!std::ptr::eq(&*evaluator.scopes[1], &*scope));
    assert!(!evaluator.scopes[1].builder_index_initialized());
    assert!(!scope.contains_key("missing"));
    evaluator.scopes[0].insert("builder".into(), Value::Unknown);
    evaluator.refresh_captured_bindings();
    assert!(evaluator.scopes[1]["builder"] == Value::Unknown);
    assert!(matches!(scope["builder"], Value::Prefix(_, _, Some(7))));
}

#[test]
fn compacted_binding_indexes_rebuild_readers_only_when_ownership_changes() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    evaluator.scopes = vec![Scope::default(), Scope::default()];
    evaluator
        .captured_bindings
        .insert(1, FxHashMap::from_iter([("builder".into(), 0)]));
    evaluator.rebuild_binding_indexes();
    let readers = evaluator.captured_binding_readers.clone();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    evaluator.rebuild_compacted_binding_indexes(false);
    assert_eq!(evaluator.captured_binding_readers, readers);
    assert!(!observer
        .snapshot()
        .work
        .contains_key("postgres.annotation.captured_reader_rebuilds"));
    evaluator
        .captured_bindings
        .insert(0, FxHashMap::from_iter([("builder".into(), 1)]));
    evaluator.rebuild_compacted_binding_indexes(true);
    assert_eq!(
        observer.snapshot().work["postgres.annotation.captured_reader_rebuilds"],
        1
    );
    let mut targets = evaluator.captured_write_targets(0, "builder");
    targets.sort_unstable();
    assert_eq!(targets, vec![0, 1]);
    // Full restoration must rebuild even after a previous no-change compaction.
    evaluator.captured_bindings.clear();
    evaluator.rebuild_binding_indexes();
    assert!(evaluator.captured_binding_readers.is_empty());
    assert_eq!(
        observer.snapshot().work["postgres.annotation.captured_reader_rebuilds"],
        2
    );
}
