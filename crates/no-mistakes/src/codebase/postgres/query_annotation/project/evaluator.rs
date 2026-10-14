use super::super::evaluate::{Evaluator, File};
use crate::fx::FxHashMap;
use std::path::PathBuf;

pub(super) fn new<'a, F>(
    files: &'a FxHashMap<PathBuf, File<'a>>,
    resolve: F,
    untrusted_array_intrinsics: bool,
) -> Evaluator<'a, F> {
    Evaluator {
        files,
        resolve,
        untrusted_array_intrinsics,
        events: Default::default(),
        scopes: Vec::new(),
        modules: crate::fx::fx_map(),
        active_module_initials: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        next_builder: 0,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: Default::default(),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        mapped_parameter_indices: Default::default(),
        deleted_argument_slots: Default::default(),
        mapped_arguments: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
    }
}
