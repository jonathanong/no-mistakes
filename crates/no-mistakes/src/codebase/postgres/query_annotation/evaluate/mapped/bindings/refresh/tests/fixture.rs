use super::*;
use crate::codebase::postgres::query_annotation::evaluate::File;
pub(super) type TestEvaluator<'a> = Evaluator<'a, fn(&str, &Path) -> Option<PathBuf>>;
pub(super) fn evaluator<'a>(files: &'a FxHashMap<PathBuf, File<'a>>) -> TestEvaluator<'a> {
    Evaluator {
        files,
        resolve: |_, _| None,
        events: Default::default(),
        scopes: Vec::new(),
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
pub(super) fn scope(values: &[(&str, Value)]) -> Scope {
    values
        .iter()
        .map(|(name, value)| ((*name).into(), value.clone()))
        .collect::<FxHashMap<_, _>>()
        .into()
}
pub(super) fn capture(evaluator: &mut TestEvaluator<'_>, frame: usize, origins: &[(&str, usize)]) {
    evaluator.captured_bindings.insert(
        frame,
        origins
            .iter()
            .map(|(name, origin)| ((*name).into(), *origin))
            .collect(),
    );
}
fn legacy(evaluator: &mut TestEvaluator<'_>) {
    for (frame, bindings) in &evaluator.captured_bindings {
        for (name, origin) in bindings {
            let value = evaluator.scopes[*origin]
                .get(name)
                .unwrap_or(&Value::Unknown);
            if evaluator.scopes[*frame].get(name) != Some(value) {
                let value = value.clone();
                evaluator.scopes[*frame].insert(name.clone(), value);
            }
        }
    }
}
fn same_state(old: &TestEvaluator<'_>, new: &TestEvaluator<'_>) {
    macro_rules! equal { ($($field:ident),+ $(,)?) => { $(assert!(old.$field == new.$field, "{}", stringify!($field));)+ }; }
    equal!(
        scopes,
        events,
        modules,
        active_callback_functions,
        active_callback_executions,
        untrusted_array_intrinsics,
        next_builder,
        invalidated_builders,
        builder_updates,
        captured_bindings,
        captured_binding_readers,
        mapped_argument_owners,
        mapped_parameter_indices,
        deleted_argument_slots,
        mapped_arguments,
        fresh_mapped_parameters,
        fresh_mapped_argument_bindings,
        argument_objects,
        argument_extra_slots,
        definite_deleted_argument_slots,
        recreated_argument_slots,
        disconnected_argument_slots
    );
    assert!(old.active_module_initials.is_empty() && new.active_module_initials.is_empty());
    assert!(old.transient_roots.is_empty() && new.transient_roots.is_empty());
}
pub(super) fn compare(old: &mut TestEvaluator<'_>, new: &mut TestEvaluator<'_>) -> usize {
    legacy(old);
    let mut names = 0;
    new.refresh_captured_bindings_with_visit(|()| names += 1);
    same_state(old, new);
    names
}
