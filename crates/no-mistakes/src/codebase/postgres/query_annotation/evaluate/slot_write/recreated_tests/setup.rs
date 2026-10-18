use crate::codebase::postgres::query_annotation::evaluate::{Evaluator, File};
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

type Resolver = fn(&str, &Path) -> Option<PathBuf>;

pub(super) fn evaluator<'a>(files: &'a FxHashMap<PathBuf, File<'a>>) -> Evaluator<'a, Resolver> {
    fn unresolved(_: &str, _: &Path) -> Option<PathBuf> {
        None
    }

    Evaluator {
        files,
        resolve: unresolved,
        events: Default::default(),
        transactions: Default::default(),
        scopes: Default::default(),
        modules: Default::default(),
        active_module_initials: Default::default(),
        transient_roots: Default::default(),
        active_callback_functions: Default::default(),
        active_callback_executions: Default::default(),
        untrusted_array_intrinsics: false,
        next_builder: 0,
        invalidated_builders: Default::default(),
        builder_updates: Default::default(),
        captured_bindings: Default::default(),
        captured_binding_readers: Default::default(),
        mapped_argument_owners: Default::default(),
        mapped_parameter_indices: Default::default(),
        deleted_argument_slots: Default::default(),
        argument_objects: Default::default(),
        argument_extra_slots: Default::default(),
        definite_deleted_argument_slots: Default::default(),
        recreated_argument_slots: Default::default(),
        disconnected_argument_slots: Default::default(),
        fresh_mapped_parameters: Default::default(),
        fresh_mapped_argument_bindings: Default::default(),
        mapped_arguments: Default::default(),
    }
}
