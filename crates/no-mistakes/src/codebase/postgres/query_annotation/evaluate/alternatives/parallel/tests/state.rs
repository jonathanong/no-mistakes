use super::*;
pub(super) fn equal<F>(serial: &Evaluator<'_, F>, parallel: &Evaluator<'_, F>) {
    assert!(
        serial.untrusted_array_intrinsics == parallel.untrusted_array_intrinsics,
        "state mismatch: untrusted_array_intrinsics"
    );
    assert!(serial.events == parallel.events, "state mismatch: events");
    assert!(serial.scopes == parallel.scopes, "state mismatch: scopes");
    assert!(
        serial.active_callback_functions == parallel.active_callback_functions,
        "state mismatch: active_callback_functions"
    );
    assert!(
        serial.active_callback_executions == parallel.active_callback_executions,
        "state mismatch: active_callback_executions"
    );
    assert!(
        serial.next_builder == parallel.next_builder,
        "state mismatch: next_builder"
    );
    assert!(
        serial.invalidated_builders == parallel.invalidated_builders,
        "state mismatch: invalidated_builders"
    );
    assert!(
        serial.builder_updates == parallel.builder_updates,
        "state mismatch: builder_updates"
    );
    assert!(
        serial.captured_bindings == parallel.captured_bindings,
        "state mismatch: captured_bindings"
    );
    assert!(
        serial.captured_binding_readers == parallel.captured_binding_readers,
        "state mismatch: captured_binding_readers"
    );
    assert!(
        serial.mapped_argument_owners == parallel.mapped_argument_owners,
        "state mismatch: mapped_argument_owners"
    );
    assert!(
        serial.mapped_parameter_indices == parallel.mapped_parameter_indices,
        "state mismatch: mapped_parameter_indices"
    );
    assert!(
        serial.deleted_argument_slots == parallel.deleted_argument_slots,
        "state mismatch: deleted_argument_slots"
    );
    assert!(
        serial.argument_objects == parallel.argument_objects,
        "state mismatch: argument_objects"
    );
    assert!(
        serial.argument_extra_slots == parallel.argument_extra_slots,
        "state mismatch: argument_extra_slots"
    );
    assert!(
        serial.definite_deleted_argument_slots == parallel.definite_deleted_argument_slots,
        "state mismatch: definite_deleted_argument_slots"
    );
    assert!(
        serial.recreated_argument_slots == parallel.recreated_argument_slots,
        "state mismatch: recreated_argument_slots"
    );
    assert!(
        serial.disconnected_argument_slots == parallel.disconnected_argument_slots,
        "state mismatch: disconnected_argument_slots"
    );
    assert!(
        serial.fresh_mapped_parameters == parallel.fresh_mapped_parameters,
        "state mismatch: fresh_mapped_parameters"
    );
    assert!(
        serial.fresh_mapped_argument_bindings == parallel.fresh_mapped_argument_bindings,
        "state mismatch: fresh_mapped_argument_bindings"
    );
    assert!(
        serial.mapped_arguments == parallel.mapped_arguments,
        "state mismatch: mapped_arguments"
    );
    assert!(serial.modules == parallel.modules);
    assert!(serial.active_module_initials.is_empty() && parallel.active_module_initials.is_empty());
    assert!(serial.transient_roots == parallel.transient_roots);
}
