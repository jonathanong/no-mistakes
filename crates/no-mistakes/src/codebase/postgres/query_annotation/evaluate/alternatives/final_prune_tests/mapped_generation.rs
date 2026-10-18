use super::lifecycle::{cycle, fixture};
use super::*;
use crate::codebase::postgres::query_annotation::evaluate::{temporaries::TransientRoot, Value};

#[test]
fn mapped_carriers_survive_only_their_live_frame_and_reused_slots_reject_stale_handles() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    let fixture = fixture();
    cycle(&mut evaluator, &fixture);
    let captured_name = fixture.alias_names[0].clone();
    let scalar = Value::Prefix(fixture.extra_prefix.clone(), true, None);
    evaluator.scopes[0].insert(captured_name.clone(), scalar.clone());
    evaluator.initialize_reference_index_for_tests();
    let old_scope = evaluator.scopes[0].clone();
    let base = evaluator.scopes.checkpoint();
    let id = evaluator.next_builder;
    evaluator.next_builder += 1;
    evaluator.create_argument_object(id, vec![scalar.clone()]);
    let frame = evaluator.environment(FxHashMap::from_iter([
        (fixture.slot_name.clone(), scalar.clone()),
        (captured_name.clone(), scalar.clone()),
    ]));
    evaluator
        .mapped_arguments
        .insert(frame, vec![(id, vec![fixture.slot_name.clone()])]);
    evaluator.rebuild_mapped_argument_owners();
    evaluator
        .fresh_mapped_parameters
        .insert(frame, FxHashSet::from_iter([fixture.slot_name.clone()]));
    evaluator.index_mapped_freshness(frame);
    evaluator.register_mapping_references(frame);
    evaluator
        .captured_bindings
        .insert(frame, FxHashMap::from_iter([(captured_name, 0)]));
    evaluator.rebuild_captured_readers();

    // The formal contains only a scalar and no `arguments` identifier exists.
    // The separate sloppy-parameter mapping is the carrier's only young root.
    evaluator.collect_final_generation(base, id, &[TransientRoot::Environment(frame)], &[]);
    assert!(evaluator.scopes.get(frame).is_some());
    assert!(evaluator.argument_objects[&id] == [scalar.clone()]);
    assert_eq!(
        evaluator.mapped_slot_target(id, 0),
        Some((frame, fixture.slot_name.clone()))
    );
    assert!(evaluator.fresh_mapped_argument_bindings[&id][&frame].contains(&fixture.slot_name));
    assert!(evaluator.captured_binding_readers[&0]
        .values()
        .any(|readers| readers.contains(&frame)));

    evaluator.collect_final_generation(base, id, &[], &[]);
    assert!(evaluator.scopes.get(frame).is_none());
    assert!(!evaluator.argument_objects.contains_key(&id));
    assert!(!evaluator.mapped_arguments.contains_key(&frame));
    assert!(!evaluator.mapped_argument_owners.contains_key(&id));
    assert!(!evaluator.mapped_parameter_indices.contains_key(&id));
    assert!(!evaluator.fresh_mapped_parameters.contains_key(&frame));
    assert!(!evaluator.fresh_mapped_argument_bindings.contains_key(&id));
    assert!(!evaluator.captured_bindings.contains_key(&frame));
    assert!(!evaluator.captured_binding_readers.contains_key(&0));
    assert!(evaluator.scopes[0] == old_scope);

    let replacement =
        evaluator.environment(FxHashMap::from_iter([(fixture.slot_name.clone(), scalar)]));
    assert_ne!(
        replacement, frame,
        "a reused physical slot receives a new handle"
    );
    assert!(
        evaluator.scopes.get_mut(frame).is_none(),
        "a stale handle cannot mutate the replacement"
    );
    assert!(evaluator.scopes.get_mut(replacement).is_some());
    let allocated = evaluator.scopes.allocated_since(base).collect::<Vec<_>>();
    assert_eq!(
        allocated,
        vec![replacement],
        "stale allocation-log entries are filtered"
    );
}
