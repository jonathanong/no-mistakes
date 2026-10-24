use super::super::{Evaluator, File, Scope, Value};
use super::scope_skip::{changed_argument_ids, unchanged_shared_scope};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::query_annotation::{Expr, QueryAnnotationFileFacts};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::diagnostics::{with_observer, InvocationObserver};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;
use std::sync::Arc;

fn prefix(id: u64) -> Value {
    Value::Prefix(format!("SELECT {id}"), true, Some(id))
}

#[test]
fn shared_scope_skips_until_its_argument_arena_changes() {
    let shared = Scope::from(FxHashMap::from_iter(
        (0..32).map(|id| (format!("sql{id}"), prefix(id))),
    ));
    let detached = shared.clone();
    let mut written = shared.clone();
    written.insert("sql0".into(), Value::Unknown);
    assert!(unchanged_shared_scope(
        &shared,
        &detached,
        &FxHashSet::default()
    ));
    assert!(!unchanged_shared_scope(
        &shared,
        &written,
        &FxHashSet::default()
    ));
    let argument = Scope::from(FxHashMap::from_iter([("args".into(), Value::Arguments(4))]));
    assert!(unchanged_shared_scope(
        &argument,
        &argument.clone(),
        &FxHashSet::from_iter([9])
    ));
    assert!(!unchanged_shared_scope(
        &argument,
        &argument.clone(),
        &FxHashSet::from_iter([4])
    ));
    let before = super::super::ArgumentObjects::from_iter([(4, vec![prefix(4)])]);
    let after = before.clone();
    assert!(changed_argument_ids(
        &before,
        &after,
        &FxHashMap::default(),
        &FxHashMap::default()
    )
    .is_empty());
    let mut replaced = before.clone();
    replaced.insert(4, vec![Value::Unknown]);
    assert_eq!(
        changed_argument_ids(
            &before,
            &replaced,
            &FxHashMap::default(),
            &FxHashMap::default()
        ),
        FxHashSet::from_iter([4])
    );
    let nested_before = super::super::ArgumentObjects::from_iter([
        (1, vec![Value::Arguments(2)]),
        (2, vec![prefix(2)]),
    ]);
    let mut nested_after = nested_before.clone();
    nested_after.insert(2, vec![prefix(9)]);
    let nested = changed_argument_ids(
        &nested_before,
        &nested_after,
        &FxHashMap::default(),
        &FxHashMap::default(),
    );
    assert!(nested.contains(&1) && nested.contains(&2));
    let holder = Scope::from(FxHashMap::from_iter([(
        "holder".into(),
        Value::Arguments(1),
    )]));
    assert!(!unchanged_shared_scope(&holder, &holder.clone(), &nested));
}

#[test]
fn local_arms_do_not_rescan_an_untouched_reference_scope() {
    let path = PathBuf::from("root.mts");
    let facts = QueryAnnotationFileFacts::default();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: Arc::new(Module::new(TsFileFacts::default().into())),
            executors: Default::default(),
        },
    )]);
    let mut evaluator = Evaluator {
        files: &files,
        resolve: |_: &str, _: &std::path::Path| -> Option<PathBuf> { None },
        events: Default::default(),
        scopes: vec![Scope::from(FxHashMap::from_iter(
            (0..200).map(|id| (format!("imported{id}"), prefix(id))),
        ))],
        modules: Default::default(),
        active_module_initials: Default::default(),
        transient_roots: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        untrusted_array_intrinsics: false,
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
    };
    let arms = [Expr::Number(1), Expr::Number(2)];
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        let env = 0;
        evaluator.alternatives(&arms, &path, &env, 16, false);
    });
    let compares = observer
        .snapshot()
        .work
        .get("postgres.annotation.prefix_compare")
        .copied()
        .unwrap_or(0);
    assert_eq!(
        compares, 0,
        "an arm that does not write the shared scope must not compare its prefixes"
    );
}
