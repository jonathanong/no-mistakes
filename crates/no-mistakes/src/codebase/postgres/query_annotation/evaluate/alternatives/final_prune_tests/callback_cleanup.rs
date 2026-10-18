use super::lifecycle::{cycle, fixture};
use super::*;
use crate::codebase::postgres::query_annotation::evaluate::transactions::Cell;

#[test]
fn callback_membership_does_not_keep_a_young_frame_alive_or_change_live_keys() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    let fixture = fixture();
    cycle(&mut evaluator, &fixture);
    evaluator.initialize_reference_index_for_tests();
    let base = evaluator.scopes.checkpoint();
    let dead = evaluator.environment(FxHashMap::default());
    let key = |env| {
        (
            PathBuf::from(&fixture.module_path),
            fixture.extra_slot as u32,
            env,
            fixture.alias_names.clone(),
        )
    };
    let live_key = key(0);
    let dead_key = key(dead);
    evaluator.set_transaction_flag(Cell::CallbackEnabled, true);
    evaluator.set_transaction_flag(Cell::ExecutionsEnabled, true);
    for key in [&live_key, &dead_key] {
        evaluator.set_transaction_flag(Cell::CallbackSeen(key.clone()), true);
        evaluator.set_transaction_flag(Cell::CallbackRunning(key.clone()), true);
    }
    evaluator.collect_final_generation(base, fixture.old_identity_cutoff, &[], &[]);

    assert!(
        evaluator.scopes.get(dead).is_none(),
        "seen/running keys are metadata, not roots"
    );
    assert!(evaluator.scopes.get(0).is_some());
    let expected = FxHashSet::from_iter([live_key]);
    assert!(evaluator.active_callback_functions.as_ref() == Some(&expected));
    assert!(evaluator.active_callback_executions.as_ref() == Some(&expected));
}
