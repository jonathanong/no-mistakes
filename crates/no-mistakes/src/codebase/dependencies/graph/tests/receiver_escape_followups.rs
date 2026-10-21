use super::receiver_escapes::{assert_export, assert_unknown, graph};

fn unknown_replica(file: &str) {
    let (built, root) = graph();
    assert_unknown(&built, &root, file, "replica.get");
    assert_export(&built, &root, file, "unrelated.get");
}

fn unknown_replica_keeps_primary(file: &str) {
    let (built, root) = graph();
    assert_unknown(&built, &root, file, "replica.get");
    assert_export(&built, &root, file, "primary.get");
    assert_export(&built, &root, file, "unrelated.get");
}

#[test]
fn class_method_this_rebind_reaches_the_argument() {
    unknown_replica("class-method-this");
}

#[test]
fn subclass_method_reaches_the_inherited_parameter() {
    unknown_replica("class-method-subclass");
}

#[test]
fn const_instance_alias_keeps_the_class() {
    unknown_replica("class-method-alias");
}

#[test]
fn instance_stored_on_an_object_keeps_the_class() {
    unknown_replica("class-method-holder");
}

#[test]
fn class_field_arrow_is_a_method_provider() {
    unknown_replica("class-method-field");
}

#[test]
fn factory_return_is_an_instance() {
    unknown_replica("class-method-factory");
}

#[test]
fn computed_method_uses_the_literal_name() {
    unknown_replica("class-method-computed");
}

#[test]
fn method_alias_call_shifts_off_the_this_argument() {
    unknown_replica("class-method-call");
}

#[test]
fn member_call_shifts_off_the_this_argument() {
    unknown_replica("class-method-member-call");
}

#[test]
fn logical_and_keeps_the_left_receiver() {
    unknown_replica("logical-and-left");
}

#[test]
fn logical_or_keeps_the_left_receiver() {
    unknown_replica("logical-or-left");
}

#[test]
fn nullish_coalesce_keeps_the_left_receiver() {
    unknown_replica("logical-nullish-left");
}

#[test]
fn logical_or_assign_includes_the_right_hand_receiver() {
    unknown_replica_keeps_primary("logical-or-assign");
}

#[test]
fn logical_and_assign_includes_the_right_hand_receiver() {
    unknown_replica_keeps_primary("logical-and-assign");
}

#[test]
fn reduce_right_element_is_the_second_parameter() {
    unknown_replica("reduce-right");
}

#[test]
fn push_before_a_spread_is_an_element() {
    unknown_replica("array-push-spread");
}

#[test]
fn push_before_for_each_is_an_element() {
    unknown_replica("array-push-foreach");
}

#[test]
fn spread_uses_the_array_before_a_later_reassignment() {
    let (built, root) = graph();
    assert_unknown(&built, &root, "array-reassign-after", "replica.get");
    assert_export(&built, &root, "array-reassign-after", "untouched.get");
    assert_export(&built, &root, "array-reassign-after", "unrelated.get");
}

#[test]
fn spread_uses_the_array_after_an_earlier_reassignment() {
    let (built, root) = graph();
    assert_export(&built, &root, "array-reassign-before", "replica.get");
    assert_unknown(&built, &root, "array-reassign-before", "untouched.get");
    assert_export(&built, &root, "array-reassign-before", "unrelated.get");
}

#[test]
fn object_assign_spread_sees_a_pushed_source() {
    // Replacing `client` and then writing `.get` may also drop `primary.get`.
    unknown_replica("object-assign-array-push");
}

#[test]
fn named_array_conditional_element_reaches_both_arms() {
    unknown_replica("named-conditional-spread");
}

#[test]
fn for_each_conditional_element_reaches_both_arms() {
    unknown_replica("foreach-conditional");
}

#[test]
fn higher_order_apply_passes_the_receiver() {
    unknown_replica_keeps_primary("higher-order-apply");
}

#[test]
fn arguments_alias_reaches_the_call_argument() {
    unknown_replica_keeps_primary("arguments-alias");
}

#[test]
fn arrow_arguments_belong_to_the_enclosing_function() {
    let (built, root) = graph();
    assert_unknown(&built, &root, "arrow-arguments", "replica.get");
    assert_export(&built, &root, "arrow-arguments", "untouched.get");
    assert_export(&built, &root, "arrow-arguments", "unrelated.get");
}

#[test]
fn callback_array_parameter_reads_the_element() {
    unknown_replica_keeps_primary("callback-array-slot");
}

fn guarded_array_escapes_both(file: &str) {
    let (built, root) = graph();
    assert_unknown(&built, &root, file, "replica.get");
    assert_unknown(&built, &root, file, "untouched.get");
    for callee in ["primary.get", "replica.info", "unrelated.get"] {
        assert_export(&built, &root, file, callee);
    }
}

#[test]
fn conditional_array_reassignment_keeps_both_origins_unknown() {
    guarded_array_escapes_both("array-reassign-conditional");
}

#[test]
fn loop_array_reassignment_keeps_both_origins_unknown() {
    guarded_array_escapes_both("array-reassign-loop");
}

#[test]
fn conditional_array_push_keeps_both_origins_unknown() {
    guarded_array_escapes_both("array-push-conditional");
}

#[test]
fn loop_array_push_keeps_both_origins_unknown() {
    guarded_array_escapes_both("array-push-loop");
}

#[test]
fn super_method_rebind_uses_the_base_instead_of_the_override() {
    let (built, root) = graph();
    assert_unknown(&built, &root, "class-method-super", "replica.get");
    for callee in ["primary.info", "replica.info", "unrelated.get"] {
        assert_export(&built, &root, "class-method-super", callee);
    }
}

#[test]
fn guarded_array_unions_slots_by_position() {
    let (built, root) = graph();
    // Slot 0 is equal on both paths; slot 1 exists only on the guarded path.
    assert_unknown(&built, &root, "array-reassign-union", "replica.get");
    assert_unknown(&built, &root, "array-reassign-union", "untouched.get");
    for callee in ["primary.get", "unrelated.get"] {
        assert_export(&built, &root, "array-reassign-union", callee);
    }
}
