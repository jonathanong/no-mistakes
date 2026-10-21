use super::*;
use crate::fx::{FxHashMap, FxHashSet};
pub(super) fn metadata<F>(fork: &mut Evaluator<'_, F>, env: usize, id: u64) {
    // These are valid typed private heap edges rooted in a real saved invocation.
    // The closed grammar cannot generate builders yet; the protocol must still
    // translate their namespace completely before any future eligibility widening.
    for (frame, object) in [(0, 0), (env, id)] {
        fork.invalidated_builders.insert(object + 1);
        fork.builder_updates.insert(
            object + 1,
            Value::Prefix("SELECT 701".into(), true, Some(object + 1)),
        );
        fork.captured_bindings.insert(
            frame,
            FxHashMap::from_iter([("pack".into(), 0), ("slots".into(), frame)]).into(),
        );
        fork.mapped_argument_owners.insert(object, frame);
        fork.mapped_parameter_indices
            .insert(object, FxHashMap::from_iter([("sql".into(), 0)]));
        fork.deleted_argument_slots
            .extend([(object, None), (object, Some(0))]);
        fork.definite_deleted_argument_slots.insert((object, 1));
        fork.recreated_argument_slots.insert((object, 2));
        fork.disconnected_argument_slots.insert((object, 3));
        fork.fresh_mapped_parameters
            .insert(frame, FxHashSet::from_iter(["sql".into()]));
        fork.fresh_mapped_argument_bindings.insert(
            object,
            FxHashMap::from_iter([(frame, FxHashSet::from_iter(["sql".into()]))]),
        );
        fork.mapped_arguments
            .insert(frame, vec![(object, vec!["sql".into()])]);
        fork.argument_extra_slots
            .entry(object)
            .or_default()
            .insert(9, Value::Arguments(object));
    }
    fork.argument_objects
        .insert(id, vec![Value::Arguments(id), Value::Primitive]);
    fork.captured_binding_readers
        .entry(0)
        .or_default()
        .insert("roundtrip".into(), FxHashSet::from_iter([0, env]));
    fork.captured_binding_readers
        .entry(env)
        .or_default()
        .insert("slots".into(), FxHashSet::from_iter([env]));
    fork.scopes[env].insert("privateArray".into(), Value::Array(id + 2));
    fork.scopes[env].insert("sql".into(), Value::Arguments(id));
    fork.argument_objects.insert(id + 2, vec![Value::Primitive]);
    // Two allocated-and-discarded IDs must remain holes after installation.
    fork.next_builder = id + 5;
}
