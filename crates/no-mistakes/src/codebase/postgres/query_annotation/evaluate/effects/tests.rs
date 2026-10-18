mod original_tests;
include!("tests/mutable_cycles.rs");
use super::super::{Evaluator, Value};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

fn evaluator<'a>(
    files: &'a FxHashMap<PathBuf, super::super::File<'a>>,
    scope: FxHashMap<String, Value>,
) -> Evaluator<'a, fn(&str, &Path) -> Option<PathBuf>> {
    let mut evaluator: Evaluator<'a, fn(&str, &Path) -> Option<PathBuf>> = Evaluator {
        files,
        resolve: |_name, _path| None,
        events: Default::default(),
        transactions: Default::default(),
        scopes: vec![scope.into()].into(),
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
    evaluator.initialize_reference_index_for_tests();
    evaluator
}

fn fixture() -> serde_json::Value {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("shared-builder-dag.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn ids(value: &serde_json::Value, name: &str) -> u64 {
    value[name].as_u64().unwrap()
}
fn shared_roots(spec: &serde_json::Value) -> Vec<Value> {
    let mut value = Value::Prefix("SELECT 1".into(), false, Some(ids(spec, "builder")));
    for _ in 0..ids(spec, "depth") {
        value = Value::Object(vec![value.clone(); ids(spec, "fanout") as usize].into());
    }
    vec![value; ids(spec, "roots") as usize]
}
fn collect_old(
    e: &Evaluator<'_, fn(&str, &Path) -> Option<PathBuf>>,
    values: &[Value],
) -> (FxHashSet<u64>, usize) {
    let mut ids = FxHashSet::default();
    let mut visits = 0;
    for value in values {
        original_tests::collect(e, value, &mut ids, &mut visits);
    }
    (ids, visits)
}
fn collect_new(
    e: &Evaluator<'_, fn(&str, &Path) -> Option<PathBuf>>,
    values: &[Value],
) -> (FxHashSet<u64>, usize) {
    let mut visits = 0;
    let ids = e.collect_builder_ids(values, |()| visits += 1);
    (ids, visits)
}
fn same_state(
    old: &Evaluator<'_, fn(&str, &Path) -> Option<PathBuf>>,
    new: &Evaluator<'_, fn(&str, &Path) -> Option<PathBuf>>,
) {
    macro_rules! equal { ($($field:ident),+ $(,)?) => { $(assert!(old.$field == new.$field, "{}", stringify!($field));)+ }; }
    let old_frames = old.scopes.ids().collect::<FxHashSet<_>>();
    let new_frames = new.scopes.ids().collect::<FxHashSet<_>>();
    assert_eq!(old_frames, new_frames, "scope handles");
    for env in old_frames {
        assert!(old.scopes[env] == new.scopes[env], "scope {env}");
    }
    equal!(
        events,
        modules,
        active_callback_functions,
        active_callback_executions,
        untrusted_array_intrinsics,
        next_builder,
        invalidated_builders,
        builder_updates,
        captured_bindings,
        captured_binding_readers,
        mapped_argument_owners,
        mapped_parameter_indices,
        deleted_argument_slots,
        mapped_arguments,
        fresh_mapped_parameters,
        fresh_mapped_argument_bindings,
        argument_objects,
        argument_extra_slots,
        definite_deleted_argument_slots,
        recreated_argument_slots,
        disconnected_argument_slots
    );
    assert!(old.active_module_initials.is_empty() && new.active_module_initials.is_empty());
    assert!(old.transient_roots.is_empty() && new.transient_roots.is_empty());
}
#[test]
fn shared_physical_dag_is_collected_once_with_full_invalidation_parity() {
    let spec = fixture();
    let values = shared_roots(&spec);
    let files = FxHashMap::default();
    let scope = FxHashMap::from_iter([(
        "builder".into(),
        Value::Prefix("SELECT 1".into(), false, Some(ids(&spec, "builder"))),
    )]);
    let (mut old, mut new) = (evaluator(&files, scope.clone()), evaluator(&files, scope));
    let (old_ids, old_visits) = collect_old(&old, &values);
    let (new_ids, new_visits) = collect_new(&new, &values);
    assert_eq!(old_ids, new_ids);
    assert_eq!(new_visits, ids(&spec, "depth") as usize);
    assert_eq!(
        old_visits,
        ids(&spec, "roots") as usize * (3usize.pow(ids(&spec, "depth") as u32) - 1) / 2
    );
    assert!(
        old_visits > new_visits * 100,
        "saved DAG must expose repeated physical nodes"
    );
    old.invalidate_builder_ids(old_ids);
    new.invalidate_builders(&values);
    same_state(&old, &new);
    assert!(new.scopes[0]["builder"] == Value::Unknown);
    assert!(new.invalidated_builders.contains(&ids(&spec, "builder")));
}
#[test]
fn every_container_wrapper_and_cow_divergence_preserve_builder_sets() {
    let files = FxHashMap::default();
    let e = evaluator(&files, FxHashMap::default());
    let spec = fixture();
    let leaf = shared_roots(&spec).remove(0);
    let shared: super::super::value::Values = vec![leaf.clone(), Value::Unknown].into();
    let mut changed = Value::Object(shared.clone());
    if let Value::Object(values) = &mut changed {
        values.push(Value::Prefix("SELECT 2".into(), false, Some(10)));
    }
    let values = vec![
        Value::Aggregate(shared.clone()),
        Value::References(shared.clone()),
        Value::Object(shared.clone()),
        Value::Record(shared.clone(), vec!["a".into(), "b".into()]),
        Value::Joined(shared.clone()),
        Value::Possible(shared),
        changed,
        Value::Promise(Box::new(leaf.clone())),
        Value::Evaluated(Box::new(leaf), false),
        Value::Number(3),
    ];
    assert_eq!(collect_old(&e, &values).0, collect_new(&e, &values).0);
    assert!(collect_new(&e, &values).0.contains(&10));
    assert_eq!(collect_new(&e, &[Value::Unknown, Value::Number(1)]).1, 0);
    assert_eq!(collect_new(&e, &[]).1, 0);
}
