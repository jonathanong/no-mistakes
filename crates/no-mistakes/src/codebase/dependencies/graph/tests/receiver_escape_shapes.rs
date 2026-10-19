use super::receiver_escapes::{assert_export, assert_unknown, graph};
use super::*;

fn unknown_replica(file: &str) {
    let (built, root) = graph();
    assert_unknown(&built, &root, file, "replica.get");
    assert_export(&built, &root, file, "unrelated.get");
}

#[test]
fn spread_element_ten_is_not_sorted_into_parameter_two() {
    unknown_replica("spread-eleven");
}

#[test]
fn argument_after_a_spread_keeps_its_shifted_slot() {
    let (built, root) = graph();
    unknown_replica_on(&built, &root, "spread-then-argument");
    assert_export(&built, &root, "spread-then-argument", "primary.get");
}

fn unknown_replica_on(built: &DepGraph, root: &std::path::Path, file: &str) {
    assert_unknown(built, root, file, "replica.get");
    assert_export(built, root, file, "unrelated.get");
}

#[test]
fn named_spread_uses_the_array_elements() {
    unknown_replica("spread-named");
}

#[test]
fn named_object_assign_spread_is_a_provider() {
    // Replacing `client` and then writing `.get` may also drop `primary.get`.
    unknown_replica("object-assign-named-spread");
}

#[test]
fn nested_conditional_argument_reaches_both_leaves() {
    unknown_replica("conditional-nested");
}

#[test]
fn class_method_conditional_argument_is_not_one_opaque_value() {
    unknown_replica("class-method-conditional");
}

#[test]
fn class_method_spread_uses_the_element() {
    unknown_replica("class-method-spread");
}

#[test]
fn reduce_without_an_initial_value_uses_the_first_element() {
    unknown_replica("reduce-no-initial");
}

#[test]
fn logical_argument_includes_both_operands() {
    unknown_replica("logical-arg");
}

#[test]
fn array_callbacks_pass_the_element() {
    unknown_replica("array-callbacks");
}

#[test]
fn reduce_element_is_the_second_parameter() {
    unknown_replica("reduce-element");
}

#[test]
fn open_tail_after_an_unresolved_spread_reaches_later_parameters() {
    let (built, root) = graph();
    unknown_replica_on(&built, &root, "open-tail");
    assert_export(&built, &root, "open-tail", "primary.get");
}

#[test]
fn sequence_argument_keeps_the_last_operands() {
    let (built, root) = graph();
    unknown_replica_on(&built, &root, "sequence-arg");
    assert_export(&built, &root, "sequence-arg", "primary.get");
}

#[test]
fn reduce_on_an_unresolved_receiver_does_not_invent_an_accumulator() {
    let (built, root) = graph();
    for callee in ["replica.get", "primary.get", "unrelated.get"] {
        assert_export(&built, &root, "reduce-unresolved", callee);
    }
}

#[test]
fn destructured_reduce_accumulator_still_passes_the_element() {
    let (built, root) = graph();
    unknown_replica_on(&built, &root, "reduce-pattern");
    assert_export(&built, &root, "reduce-pattern", "primary.get");
}

#[test]
fn unrelated_object_assign_field_drops_the_previous_callee() {
    let (built, root) = graph();
    assert_unknown(
        &built,
        &root,
        "object-assign-unrelated-read",
        "box.client.get",
    );
    assert_export(&built, &root, "object-assign-unrelated-read", "primary.get");
    assert_export(&built, &root, "object-assign-unrelated-read", "replica.get");
    assert_export(
        &built,
        &root,
        "object-assign-unrelated-read",
        "unrelated.get",
    );
}
