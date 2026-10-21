use super::*;
use crate::fx::{FxHashMap, FxHashSet};
pub(super) fn metadata<F>(target: &Evaluator<'_, F>, env: usize, id: u64) {
    assert!(target.invalidated_builders.contains(&(id + 1)));
    assert!(
        matches!(&target.builder_updates[&(id + 1)], Value::Prefix(sql, true, Some(object)) if sql == "SELECT 701" && *object == id + 1)
    );
    assert_eq!(target.captured_bindings[&env]["slots"], env);
    assert_eq!(target.captured_bindings[&env]["pack"], 0);
    assert_eq!(
        target.captured_binding_readers[&0]["roundtrip"],
        FxHashSet::from_iter([env])
    );
    assert_eq!(
        target.captured_binding_readers[&env]["slots"],
        FxHashSet::from_iter([env])
    );
    assert_eq!(target.mapped_argument_owners[&id], env);
    assert_eq!(target.mapped_parameter_indices[&id]["sql"], 0);
    assert!(target.deleted_argument_slots.contains(&(id, None)));
    assert!(target.deleted_argument_slots.contains(&(id, Some(0))));
    assert!(target.definite_deleted_argument_slots.contains(&(id, 1)));
    assert!(target.recreated_argument_slots.contains(&(id, 2)));
    assert!(target.disconnected_argument_slots.contains(&(id, 3)));
    assert_eq!(
        target.fresh_mapped_parameters[&env],
        FxHashSet::from_iter(["sql".into()])
    );
    assert_eq!(
        target.fresh_mapped_argument_bindings[&id],
        FxHashMap::from_iter([(env, FxHashSet::from_iter(["sql".into()]))])
    );
    assert_eq!(
        target.mapped_arguments[&env],
        vec![(id, vec!["sql".into()])]
    );
    assert!(
        matches!(target.argument_extra_slots[&id][&9], Value::Arguments(object) if object == id)
    );
    assert!(matches!(target.scopes[env]["privateArray"], Value::Array(object) if object == id + 2));
}
