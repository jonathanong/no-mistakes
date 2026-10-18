use super::*;
use crate::codebase::postgres::query_annotation::evaluate::temporaries::TransientRoot;
use crate::codebase::postgres::query_annotation::evaluate::{Scope, Value};
use crate::fx::{FxHashMap, FxHashSet};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub(super) struct LifecycleFixture {
    pub(super) old_identity_cutoff: u64,
    pub(super) argument_objects: Vec<ArgumentObject>,
    pub(super) builder_id: u64,
    pub(super) builder_prefix: String,
    pub(super) alias_names: Vec<String>,
    pub(super) slot_name: String,
    pub(super) module_binding: String,
    pub(super) module_path: String,
    pub(super) extra_slot: usize,
    pub(super) extra_prefix: String,
}

#[derive(Deserialize)]
pub(super) struct ArgumentObject {
    pub(super) id: u64,
    pub(super) arguments: Vec<u64>,
    pub(super) builders: Vec<u64>,
}

pub(super) fn fixture() -> LifecycleFixture {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("reference-lifecycle.json");
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

pub(super) fn cycle(evaluator: &mut TestEvaluator<'_>, fixture: &LifecycleFixture) {
    evaluator.next_builder = fixture.old_identity_cutoff;
    evaluator.scopes.allocate(Scope::default());
    for object in &fixture.argument_objects {
        let mut values = object
            .arguments
            .iter()
            .copied()
            .map(Value::Arguments)
            .collect::<Vec<_>>();
        values.extend(object.builders.iter().map(|_| {
            Value::Prefix(
                fixture.builder_prefix.clone(),
                true,
                Some(fixture.builder_id),
            )
        }));
        evaluator.argument_objects.insert(object.id, values);
    }
    evaluator.builder_updates.insert(
        fixture.builder_id,
        Value::Prefix(fixture.extra_prefix.clone(), true, None),
    );
    evaluator.invalidated_builders.insert(fixture.builder_id);
}

#[test]
fn live_alias_holds_cycle_until_the_last_root_is_removed_then_builder_metadata_dies() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    let fixture = fixture();
    cycle(&mut evaluator, &fixture);
    evaluator.scopes[0].insert(fixture.alias_names[0].clone(), Value::Arguments(1));
    evaluator.scopes[0].insert(fixture.alias_names[1].clone(), Value::Arguments(1));
    evaluator.initialize_reference_index_for_tests();

    evaluator.write_binding(0, fixture.alias_names[0].clone(), Value::Unknown);
    evaluator.reclaim_dead_state();
    assert!(evaluator.argument_objects.contains_key(&1));
    assert!(evaluator.argument_objects.contains_key(&2));
    assert!(evaluator.builder_updates.contains_key(&fixture.builder_id));

    evaluator.write_binding(0, fixture.alias_names[1].clone(), Value::Unknown);
    evaluator.reclaim_dead_state();
    assert!(!evaluator.argument_objects.contains_key(&1));
    assert!(!evaluator.argument_objects.contains_key(&2));
    assert!(!evaluator.builder_updates.contains_key(&fixture.builder_id));
    assert!(!evaluator.invalidated_builders.contains(&fixture.builder_id));
}

#[test]
fn transient_roots_keep_cycles_for_one_trial_and_module_snapshots_pin_them_until_finish() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    let fixture = fixture();
    cycle(&mut evaluator, &fixture);
    evaluator.scopes[0].insert(fixture.module_binding.clone(), Value::Arguments(1));
    evaluator.initialize_reference_index_for_tests();
    evaluator.write_binding(0, fixture.module_binding.clone(), Value::Unknown);
    evaluator.transient_roots.push(TransientRoot::Argument(1));
    evaluator.reclaim_dead_state();
    assert!(evaluator.argument_objects.contains_key(&1));
    evaluator.transient_roots.clear();
    evaluator.reclaim_dead_state();
    assert!(!evaluator.argument_objects.contains_key(&1));

    cycle(&mut evaluator, &fixture);
    evaluator.initialize_reference_index_for_tests();
    let base = evaluator.scopes.checkpoint();
    evaluator.begin_alternative_module_snapshot(base);
    let env = evaluator.scopes.allocate(Scope::default());
    evaluator.write_binding(env, fixture.module_binding.clone(), Value::Arguments(1));
    evaluator.record_alternative_module_initial(Path::new(&fixture.module_path), env);
    assert!(evaluator.identity_is_pinned(1));
    evaluator.write_binding(env, fixture.module_binding.clone(), Value::Unknown);
    evaluator.reclaim_dead_state();
    assert!(evaluator.argument_objects.contains_key(&1));
    evaluator.transient_roots = evaluator.finish_alternative_module_snapshot();
    evaluator.reclaim_dead_state();
    assert!(!evaluator.argument_objects.contains_key(&1));
}

#[test]
fn before_image_and_delta_pins_keep_old_and_new_values_alive_until_released() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    let fixture = fixture();
    cycle(&mut evaluator, &fixture);
    evaluator.scopes[0].insert(fixture.slot_name.clone(), Value::Arguments(1));
    evaluator.initialize_reference_index_for_tests();
    let checkpoint = evaluator.begin_transaction();
    evaluator.write_binding(0, fixture.slot_name.clone(), Value::Unknown);
    assert!(evaluator.identity_is_pinned(1));
    evaluator.reclaim_dead_state();
    assert!(evaluator.argument_objects.contains_key(&1));
    let delta = evaluator.finish_transaction(checkpoint);
    assert!(evaluator.scopes[0][fixture.slot_name.as_str()] == Value::Arguments(1));
    assert!(evaluator.argument_objects.contains_key(&1));
    evaluator.release_delta_pins(delta.pins);

    evaluator.write_binding(0, fixture.slot_name.clone(), Value::Unknown);
    let checkpoint = evaluator.begin_transaction();
    evaluator.write_binding(0, "slot".into(), Value::Arguments(1));
    let delta = evaluator.finish_transaction(checkpoint);
    assert_eq!(delta.pins, FxHashSet::from_iter([1]));
    evaluator.reclaim_dead_state();
    assert!(evaluator.argument_objects.contains_key(&1));
    evaluator.release_delta_pins(delta.pins);
    evaluator.reclaim_dead_state();
    assert!(!evaluator.argument_objects.contains_key(&1));
}

#[test]
fn rollback_restores_primary_tables_and_their_derived_reference_indexes() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files).with_checkpoint_index_oracle();
    let fixture = fixture();
    cycle(&mut evaluator, &fixture);
    evaluator.scopes[0].insert(fixture.slot_name.clone(), Value::Arguments(1));
    evaluator.argument_extra_slots.insert(
        1,
        std::collections::BTreeMap::from_iter([(
            fixture.extra_slot,
            Value::Prefix(fixture.extra_prefix.clone(), true, Some(fixture.builder_id)),
        )]),
    );
    evaluator.initialize_reference_index_for_tests();
    let scopes = evaluator
        .scopes
        .ids()
        .map(|env| (env, evaluator.scopes[env].clone()))
        .collect::<FxHashMap<_, _>>();
    let objects = evaluator.argument_objects.clone();
    let extras = evaluator.argument_extra_slots.clone();
    let builders = evaluator.builder_updates.clone();
    let invalidated = evaluator.invalidated_builders.clone();
    let checkpoint = evaluator.begin_transaction();
    evaluator.write_binding(0, fixture.slot_name.clone(), Value::Unknown);
    evaluator.write_dense_slot(1, 0, Value::Unknown);
    evaluator.write_sparse_slot(1, fixture.extra_slot, None);
    evaluator.write_builder(fixture.builder_id, Value::Unknown);
    let delta = evaluator.finish_transaction(checkpoint);
    assert!(
        scopes
            == evaluator
                .scopes
                .ids()
                .map(|env| (env, evaluator.scopes[env].clone()))
                .collect()
    );
    assert!(objects == evaluator.argument_objects);
    assert!(extras == evaluator.argument_extra_slots);
    assert!(builders == evaluator.builder_updates);
    assert!(invalidated == evaluator.invalidated_builders);
    evaluator.assert_checkpoint_indexes();
    evaluator.release_delta_pins(delta.pins);
}
