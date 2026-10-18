use super::lifecycle::{cycle, fixture};
use super::*;
use crate::codebase::postgres::query_annotation::evaluate::transactions::Cell;
use crate::codebase::postgres::query_annotation::evaluate::Value;

#[test]
fn completion_promotes_outer_initials_without_rebasing_an_inner_reserved_module() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    let fixture = fixture();
    cycle(&mut evaluator, &fixture);
    let first = fixture.argument_objects[0].id;
    let second = fixture.argument_objects[1].id;
    evaluator.scopes[0].insert(fixture.slot_name.clone(), Value::Arguments(first));
    evaluator.initialize_reference_index_for_tests();

    let outer_base = evaluator.scopes.checkpoint();
    evaluator.begin_alternative_module_snapshot(outer_base);
    let outer = evaluator.begin_transaction();
    let module = evaluator.environment(FxHashMap::from_iter([(
        fixture.module_binding.clone(),
        Value::Arguments(first),
    )]));
    let path = PathBuf::from(&fixture.module_path);
    evaluator.save_cell(Cell::Module(path.clone()));
    evaluator.modules.insert(path.clone(), module);

    // The same cyclic module is new to the outer arm but was already reserved
    // before the inner checkpoint. Their pristine incoming states differ.
    let inner_base = evaluator.scopes.checkpoint();
    evaluator.begin_alternative_module_snapshot(inner_base);
    let inner = evaluator.begin_transaction();
    evaluator.write_dense_slot(first, 0, Value::Arguments(first));
    evaluator.record_alternative_module_initial(&path, module);
    assert!(
        evaluator.identity_is_pinned(second),
        "the inner before-image still owns its old target"
    );

    let delta = evaluator.finish_transaction(inner);
    assert!(evaluator.argument_objects[&first][0] == Value::Arguments(second));
    evaluator.release_delta_pins(delta.pins);
    assert!(evaluator.finish_alternative_module_snapshot().is_empty());

    let delta = evaluator.finish_transaction(outer);
    assert!(
        evaluator.argument_objects[&first][0] == Value::Arguments(first),
        "outer rollback restores its completed module initial"
    );
    assert_eq!(evaluator.modules.get(&path), Some(&module));
    evaluator.release_delta_pins(delta.pins);
    evaluator.finish_alternative_module_snapshot();
    evaluator.reclaim_dead_state();
    assert!(evaluator.argument_objects.contains_key(&first));
    assert!(!evaluator.argument_objects.contains_key(&second));
}

#[test]
fn a_module_reserved_before_the_only_checkpoint_is_not_promoted() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    let fixture = fixture();
    cycle(&mut evaluator, &fixture);
    let first = fixture.argument_objects[0].id;
    let second = fixture.argument_objects[1].id;
    evaluator.scopes[0].insert(fixture.module_binding.clone(), Value::Arguments(first));
    evaluator.initialize_reference_index_for_tests();
    let base = evaluator.scopes.checkpoint();
    evaluator.begin_alternative_module_snapshot(base);
    let checkpoint = evaluator.begin_transaction();
    evaluator.write_dense_slot(first, 0, Value::Arguments(first));
    evaluator.record_alternative_module_initial(std::path::Path::new(&fixture.module_path), 0);
    let delta = evaluator.finish_transaction(checkpoint);
    assert!(evaluator.argument_objects[&first][0] == Value::Arguments(second));
    evaluator.release_delta_pins(delta.pins);
    assert!(evaluator.finish_alternative_module_snapshot().is_empty());
    assert!(!evaluator.identity_is_pinned(first));
    assert!(!evaluator.identity_is_pinned(second));
}

#[test]
fn duplicate_module_completion_does_not_add_a_second_pin_lifetime() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    let fixture = fixture();
    cycle(&mut evaluator, &fixture);
    let first = fixture.argument_objects[0].id;
    evaluator.initialize_reference_index_for_tests();
    let base = evaluator.scopes.checkpoint();
    evaluator.begin_alternative_module_snapshot(base);
    let module = evaluator.environment(FxHashMap::from_iter([(
        fixture.module_binding.clone(),
        Value::Arguments(first),
    )]));
    let path = std::path::Path::new(&fixture.module_path);
    evaluator.record_alternative_module_initial(path, module);
    evaluator.record_alternative_module_initial(path, module);
    assert!(evaluator.identity_is_pinned(first));
    assert_eq!(evaluator.finish_alternative_module_snapshot().len(), 1);
    assert!(!evaluator.identity_is_pinned(first));
}

#[test]
fn promotion_rebases_old_target_pins_to_the_initialized_builder_and_counts_its_walk() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    let fixture = fixture();
    cycle(&mut evaluator, &fixture);
    let first = fixture.argument_objects[0].id;
    let second = fixture.argument_objects[1].id;
    evaluator.scopes[0].insert(fixture.slot_name.clone(), Value::Arguments(first));
    evaluator.initialize_reference_index_for_tests();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _observer = crate::diagnostics::InvocationGuard::install(observer.clone());

    let base = evaluator.scopes.checkpoint();
    evaluator.begin_alternative_module_snapshot(base);
    let checkpoint = evaluator.begin_transaction();
    let module = evaluator.environment(FxHashMap::from_iter([(
        fixture.module_binding.clone(),
        Value::Arguments(first),
    )]));
    let path = PathBuf::from(&fixture.module_path);
    evaluator.save_cell(Cell::Module(path.clone()));
    evaluator.modules.insert(path.clone(), module);
    let initialized = Value::Prefix(
        fixture.builder_prefix.clone(),
        true,
        Some(fixture.builder_id),
    );
    evaluator.write_dense_slot(first, 0, initialized.clone());
    assert!(evaluator.identity_is_pinned(second));
    evaluator.record_alternative_module_initial(&path, module);
    // A completed initial replaces the old before-image, including its pins.
    assert!(!evaluator.identity_is_pinned(second));
    assert!(evaluator.identity_is_pinned(fixture.builder_id));
    evaluator.reclaim_dead_state();
    assert!(!evaluator.argument_objects.contains_key(&second));

    let delta = evaluator.finish_transaction(checkpoint);
    assert!(evaluator.argument_objects[&first][0] == initialized);
    assert_eq!(evaluator.modules.get(&path), Some(&module));
    evaluator.release_delta_pins(delta.pins);
    assert_eq!(evaluator.finish_alternative_module_snapshot().len(), 1);
    assert!(!evaluator.identity_is_pinned(first));
    assert!(!evaluator.identity_is_pinned(fixture.builder_id));
    let work = observer.snapshot().work;
    assert_eq!(work["postgres.annotation.module_initial_promotions"], 1);
    assert_eq!(work["postgres.annotation.full_frame_visits"], 1);
    assert_eq!(
        work["postgres.annotation.module_initial_new_frame_visits"],
        1
    );
}
