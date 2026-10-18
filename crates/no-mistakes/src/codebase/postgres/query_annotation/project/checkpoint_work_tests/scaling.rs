//! Compare real saved-source evaluation, including its final SQL and copy work.
use super::*;

const COPY_WORK: &[&str] = &[
    "postgres.annotation.undo_values_cloned",
    "postgres.annotation.undo_after_values_cloned",
    "postgres.annotation.undo_restore_values_cloned",
];
const LOCATION_WORK: &[&str] = &[
    "postgres.annotation.undo_entries_recorded",
    "postgres.annotation.undo_entries_restored",
    "postgres.annotation.join_delta_cells_visited",
    "postgres.annotation.join_apply_cells_visited",
];

fn equal_work(
    small: &crate::diagnostics::DiagnosticsSnapshot,
    large: &crate::diagnostics::DiagnosticsSnapshot,
) {
    for key in COPY_WORK.iter().chain(LOCATION_WORK) {
        assert!(work(small, key) > 0, "actual work is observable: {key}");
        assert_eq!(work(small, key), work(large, key), "{key}");
    }
}

#[test]
fn unchanged_mapped_parameters_do_not_add_binding_join_or_restore_work() {
    let (small_sql, small) = mapped("mapped-small.cjs");
    let (wide_sql, wide) = mapped("mapped-untouched-wide.cjs");
    assert_eq!(small_sql, wide_sql);
    assert_eq!(
        small_sql,
        ["SELECT 1 /* left-00 */", "SELECT 2 /* right-00 */"]
    );
    let key = "postgres.annotation.joined_binding_locations";
    assert_eq!(work(&small, key), 1);
    assert_eq!(work(&wide, key), 1);
    equal_work(&small, &wide);
}

#[test]
fn repeated_writes_save_and_restore_one_first_image_per_location() {
    let (single_call, single_sql, single) = run(&["state-single-write.mts"]);
    let (repeated_call, repeated_sql, repeated) = run(&["state-small.mts"]);
    assert_eq!(single_call, repeated_call);
    assert!(single_call.is_some());
    assert_eq!(single_sql, repeated_sql);
    assert_eq!(single_sql, Err("unresolved-value-or-effect"));
    equal_work(&single, &repeated);
}

#[test]
fn wider_untouched_dense_state_does_not_add_actual_value_copies_or_join_visits() {
    let (small_call, small_sql, small) = run(&["state-small.mts"]);
    let (wide_call, wide_sql, wide) = run(&["state-large.mts"]);
    assert_eq!(small_call, wide_call);
    assert!(small_call.is_some());
    assert_eq!(small_sql, wide_sql);
    equal_work(&small, &wide);
}
