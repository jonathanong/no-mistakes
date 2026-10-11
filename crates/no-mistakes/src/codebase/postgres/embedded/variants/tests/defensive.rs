use super::*;

#[test]
fn unsupported_mutation_and_builder_inputs_do_not_publish_partial_versions() {
    let calls = calls("variants-defensive.ts");
    assert_eq!(calls.len(), 22);
    assert!(
        calls.iter().all(EmbeddedSqlCall::is_unanalyzable),
        "{calls:#?}"
    );
}

#[test]
fn alternative_union_caps_apply_to_conditionals_as_well_as_fragment_products() {
    let calls = calls("variants-cap-alternatives.ts");
    assert_eq!(calls.len(), 1);
    assert!(calls[0].is_unanalyzable());
}

#[test]
fn known_binding_values_and_derived_predicates_preserve_reachable_paths() {
    let calls = calls("variants-condition-values.ts");
    assert_eq!(calls.len(), 9);
    for (call, expected) in calls
        .iter()
        .take(5)
        .zip(["SELECT 1", "SELECT 2", "SELECT 3", "SELECT 4", "SELECT 5"])
    {
        assert_eq!(texts(call), [expected]);
    }
    for (call, expected) in calls[5..8].iter().zip([
        ["SELECT 6", "SELECT 7"],
        ["SELECT 10", "SELECT 29"],
        ["SELECT 19", "SELECT 20"],
    ]) {
        let mut actual = texts(call);
        actual.sort();
        assert_eq!(actual, expected);
    }
    assert_eq!(texts(&calls[8]), ["SELECT 8"]);
}

#[test]
fn literal_truth_and_short_circuit_mutations_choose_only_executed_paths() {
    let calls = calls("variants-literal-conditions.ts");
    assert_eq!(calls.len(), 12);
    for (call, expected) in calls.iter().zip([
        "SELECT 1",
        "SELECT 2",
        "SELECT 3",
        "SELECT 4",
        "SELECT 5",
        "SELECT 6",
        "SELECT 7",
        "SELECT 8",
        "SELECT 9 LIMIT 1",
        "SELECT 10 LIMIT 2",
        "SELECT 11 LIMIT 3",
        "SELECT 12",
    ]) {
        assert_eq!(texts(call), [expected]);
    }
}

#[test]
fn recoverable_guards_and_unary_conditions_keep_only_complete_reachable_sql() {
    let calls = calls("variants-recovery-guards.ts");
    assert_eq!(calls.len(), 17);
    assert_eq!(texts(&calls[0]), ["SELECT 24 "]);
    assert_eq!(texts(&calls[1]), ["SELECT 25 LIMIT 1"]);
    assert_eq!(texts(&calls[2]), ["SELECT 26"]);
    assert_eq!(texts(&calls[3]), ["SELECT 27", "SELECT 28"]);
    assert!(calls[4..11].iter().all(EmbeddedSqlCall::is_unanalyzable));
    assert_eq!(texts(&calls[11]), ["SELECT 32"]);
    assert_eq!(texts(&calls[12]), ["SELECT 33"]);
    assert_eq!(texts(&calls[13]), ["SELECT 34 ", "SELECT 35"]);
    assert_eq!(texts(&calls[14]), ["SELECT 36", "SELECT 37"]);
    assert_eq!(texts(&calls[15]), ["SELECT 38", "SELECT 39"]);
    assert_eq!(texts(&calls[16]), ["SELECT 40"]);
}

#[test]
fn lazy_fallbacks_recover_the_right_operand_only_on_its_actual_left_paths() {
    let calls = calls("variants-lazy-fallbacks.ts");
    assert_eq!(calls.len(), 3);
    for (call, expected) in calls.iter().zip([
        ["SELECT 1", "SELECT 2"],
        ["SELECT 3", "SELECT 4"],
        ["SELECT 5 ", "SELECT 5 LIMIT 1"],
    ]) {
        let mut actual = texts(call);
        actual.sort();
        assert_eq!(actual, expected);
    }
}

#[test]
fn known_condition_values_do_not_hide_a_reachable_opaque_sql_arm() {
    let calls = calls("variants-condition-opaque.ts");
    assert_eq!(calls.len(), 5);
    assert!(calls.iter().all(EmbeddedSqlCall::is_unanalyzable));
}

#[test]
fn branch_snapshots_keep_only_bindings_possible_on_their_immutable_condition_paths() {
    let calls = calls("variants-snapshot-paths.ts");
    assert_eq!(calls.len(), 7);
    for (call, expected) in calls.iter().zip([
        vec!["SELECT 10", "SELECT 2"],
        vec!["SELECT 10", "SELECT 29"],
        vec!["SELECT 10", "SELECT 2"],
        vec!["SELECT 1", "SELECT 29"],
        vec!["SELECT 10", "SELECT 2"],
        vec!["SELECT 10", "SELECT 19", "SELECT 2"],
        vec!["SELECT 0", "SELECT 10", "SELECT 19", "SELECT 2", "SELECT 9"],
    ]) {
        let mut actual = texts(call);
        actual.sort();
        assert_eq!(actual, expected);
    }
}
