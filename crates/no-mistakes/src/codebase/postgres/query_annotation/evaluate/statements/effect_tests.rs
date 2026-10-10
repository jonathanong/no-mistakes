use super::super::{Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

fn evaluator<'a>(
    files: &'a FxHashMap<PathBuf, super::super::File<'a>>,
    scope: FxHashMap<String, Value>,
) -> Evaluator<'a, fn(&str, &Path) -> Option<PathBuf>> {
    Evaluator {
        files,
        resolve: |_name, _path| None,
        events: Default::default(),
        scopes: vec![scope.into()],
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
fn argument_property_reads_stay_pure_and_nested_members_do_not() {
    let mut scope = FxHashMap::default();
    scope.insert("arguments".into(), Value::Arguments(1));
    scope.insert(
        "wrapped".into(),
        Value::Evaluated(Box::new(Value::Arguments(1)), true),
    );
    scope.insert("other".into(), Value::Unknown);
    let files = FxHashMap::default();
    let evaluator = evaluator(&files, scope);
    let path = Path::new("query.mts");
    assert!(!evaluator.effect_can_mutate(
        &Expr::Index(Box::new(Expr::Name("arguments".into())), 0),
        path,
        0,
    ));
    assert!(!evaluator.effect_can_mutate(
        &Expr::Member(Box::new(Expr::Name("wrapped".into())), "length".into()),
        path,
        0,
    ));
    assert!(evaluator.effect_can_mutate(
        &Expr::Member(
            Box::new(Expr::Index(Box::new(Expr::Name("arguments".into())), 0)),
            "trigger".into(),
        ),
        path,
        0,
    ));
    assert!(evaluator.effect_can_mutate(
        &Expr::Member(Box::new(Expr::Name("other".into())), "trigger".into()),
        path,
        0,
    ));
    // A named property of the arguments object itself can be an accessor.
    assert!(evaluator.effect_can_mutate(
        &Expr::Member(Box::new(Expr::Name("arguments".into())), "trigger".into()),
        path,
        0,
    ));
    assert!(evaluator.effect_can_mutate(
        &Expr::Member(Box::new(Expr::Name("wrapped".into())), "trigger".into()),
        path,
        0,
    ));
}

#[test]
fn invalidation_without_builder_ids_preserves_shared_binding_snapshots() {
    let files = FxHashMap::default();
    let scope = FxHashMap::from_iter([
        ("unrelated".into(), Value::Unknown),
        (
            "builder".into(),
            Value::Prefix("SELECT 1".into(), false, Some(7)),
        ),
    ]);
    let mut evaluator = evaluator(&files, scope);
    let snapshot = evaluator.scopes[0].clone();
    evaluator.invalidate_builders(&[Value::Unknown]);
    // A no-op must not copy binding maps for every retained invocation frame.
    assert!(std::ptr::eq(&*snapshot, &*evaluator.scopes[0]));
    assert!(evaluator.invalidated_builders.is_empty());

    let builder = evaluator.scopes[0]["builder"].clone();
    evaluator.invalidate_builders(&[builder]);
    assert!(evaluator.scopes[0]["builder"] == Value::Unknown);
    assert!(matches!(snapshot["builder"], Value::Prefix(_, _, Some(7))));
    assert!(evaluator.invalidated_builders.contains(&7));
}
