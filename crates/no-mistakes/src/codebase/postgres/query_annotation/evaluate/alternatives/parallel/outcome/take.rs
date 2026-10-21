use super::*;
impl Outcome {
    pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives::parallel) fn take<
        F,
    >(
        mut fork: Evaluator<'_, F>,
        value: Value,
        base_env: usize,
        base_id: u64,
    ) -> Self {
        debug_assert!(fork.events.is_empty());
        fork.scopes.drain(..base_env);
        fork.invalidated_builders.retain(|id| *id >= base_id);
        fork.builder_updates.retain(|id, _| *id >= base_id);
        fork.captured_bindings.retain(|env, _| *env >= base_env);
        fork.captured_binding_readers.retain(|_, names| {
            names.retain(|_, readers| {
                readers.retain(|env| *env >= base_env);
                !readers.is_empty()
            });
            !names.is_empty()
        });
        fork.mapped_argument_owners.retain(|id, _| *id >= base_id);
        fork.mapped_parameter_indices.retain(|id, _| *id >= base_id);
        fork.deleted_argument_slots.retain(|(id, _)| *id >= base_id);
        fork.argument_objects.retain(|id, _| *id >= base_id);
        fork.argument_extra_slots.retain(|id, _| *id >= base_id);
        fork.definite_deleted_argument_slots
            .retain(|(id, _)| *id >= base_id);
        fork.recreated_argument_slots
            .retain(|(id, _)| *id >= base_id);
        fork.disconnected_argument_slots
            .retain(|(id, _)| *id >= base_id);
        fork.fresh_mapped_parameters
            .retain(|env, _| *env >= base_env);
        fork.fresh_mapped_argument_bindings
            .retain(|id, _| *id >= base_id);
        fork.mapped_arguments.retain(|env, _| *env >= base_env);
        Self {
            base_env,
            base_id,
            value,
            scopes: fork.scopes,
            next_builder: fork.next_builder,
            invalidated_builders: fork.invalidated_builders,
            builder_updates: fork.builder_updates,
            captured_bindings: fork.captured_bindings,
            captured_binding_readers: fork.captured_binding_readers,
            mapped_argument_owners: fork.mapped_argument_owners,
            mapped_parameter_indices: fork.mapped_parameter_indices,
            deleted_argument_slots: fork.deleted_argument_slots,
            argument_objects: fork.argument_objects,
            argument_extra_slots: fork.argument_extra_slots,
            definite_deleted_argument_slots: fork.definite_deleted_argument_slots,
            recreated_argument_slots: fork.recreated_argument_slots,
            disconnected_argument_slots: fork.disconnected_argument_slots,
            fresh_mapped_parameters: fork.fresh_mapped_parameters,
            fresh_mapped_argument_bindings: fork.fresh_mapped_argument_bindings,
            mapped_arguments: fork.mapped_arguments,
        }
    }
}
