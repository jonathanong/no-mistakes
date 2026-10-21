//! Metadata snapshots share Scope, DenseSlots, capture origins and immutable files.
use super::*;
impl<'a, F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'a, F> {
    pub(super) fn fork_arm(&self) -> Evaluator<'a, &F> {
        Evaluator {
            parallel_arms: None,
            files: self.files,
            resolve: &self.resolve,
            untrusted_array_intrinsics: self.untrusted_array_intrinsics,
            events: Default::default(),
            scopes: self.scopes.clone(),
            modules: self.modules.clone(),
            active_module_initials: Vec::new(),
            transient_roots: self.transient_roots.clone(),
            active_callback_functions: self.active_callback_functions.clone(),
            active_callback_executions: self.active_callback_executions.clone(),
            next_builder: self.next_builder,
            invalidated_builders: self.invalidated_builders.clone(),
            builder_updates: self.builder_updates.clone(),
            captured_bindings: self.captured_bindings.clone(),
            captured_binding_readers: self.captured_binding_readers.clone(),
            mapped_argument_owners: self.mapped_argument_owners.clone(),
            mapped_parameter_indices: self.mapped_parameter_indices.clone(),
            deleted_argument_slots: self.deleted_argument_slots.clone(),
            argument_objects: self.argument_objects.clone(),
            argument_extra_slots: self.argument_extra_slots.clone(),
            definite_deleted_argument_slots: self.definite_deleted_argument_slots.clone(),
            recreated_argument_slots: self.recreated_argument_slots.clone(),
            disconnected_argument_slots: self.disconnected_argument_slots.clone(),
            fresh_mapped_parameters: self.fresh_mapped_parameters.clone(),
            fresh_mapped_argument_bindings: self.fresh_mapped_argument_bindings.clone(),
            mapped_arguments: self.mapped_arguments.clone(),
        }
    }
}
