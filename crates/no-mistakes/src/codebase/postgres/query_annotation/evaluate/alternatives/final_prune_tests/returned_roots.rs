use super::lifecycle::{cycle, fixture};
use super::*;
use crate::codebase::postgres::query_annotation::evaluate::Value;

#[test]
fn final_collection_preserves_an_old_array_held_only_by_the_return_value() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    let fixture = fixture();
    cycle(&mut evaluator, &fixture);
    evaluator.initialize_reference_index_for_tests();
    let id = fixture.argument_objects[0].id;
    let base = evaluator.scopes.checkpoint();
    let returned = vec![Value::Array(id)];

    // The old cycle has no binding root. Young marking alone skips its IDs;
    // its returned value must also protect it during affected-component GC.
    evaluator.collect_final_generation(base, fixture.old_identity_cutoff, &[], &returned);
    assert!(fixture
        .argument_objects
        .iter()
        .all(|object| evaluator.argument_objects.contains_key(&object.id)));
    assert!(evaluator.builder_updates.contains_key(&fixture.builder_id));
    assert!(evaluator.invalidated_builders.contains(&fixture.builder_id));
    assert!(
        !evaluator.identity_is_pinned(id),
        "the temporary pin is released after collection"
    );

    drop(returned);
    evaluator.reclaim_dead_state();
    assert!(evaluator.argument_objects.is_empty());
    assert!(!evaluator.builder_updates.contains_key(&fixture.builder_id));
}

#[test]
fn final_collection_preserves_returned_prefix_metadata_without_an_array_holder() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    let fixture = fixture();
    cycle(&mut evaluator, &fixture);
    evaluator.initialize_reference_index_for_tests();
    for object in &fixture.argument_objects {
        evaluator.drop_argument_state(object.id);
    }
    let returned = vec![Value::Prefix(
        fixture.builder_prefix.clone(),
        true,
        Some(fixture.builder_id),
    )];
    let base = evaluator.scopes.checkpoint();
    evaluator.collect_final_generation(base, fixture.old_identity_cutoff, &[], &returned);
    assert!(evaluator.argument_objects.is_empty());
    assert!(evaluator.builder_updates.contains_key(&fixture.builder_id));
    assert!(evaluator.invalidated_builders.contains(&fixture.builder_id));
    assert!(!evaluator.identity_is_pinned(fixture.builder_id));

    drop(returned);
    evaluator.reclaim_dead_state();
    assert!(!evaluator.builder_updates.contains_key(&fixture.builder_id));
    assert!(!evaluator.invalidated_builders.contains(&fixture.builder_id));
}
