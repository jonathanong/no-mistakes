fn runtime_namespace_members<'a>(
    inputs: &ExportEdgeInputs<'_>,
    file_facts: &'a crate::codebase::ts_source::facts::TsFileFacts,
    source: &str,
    local: &str,
    target: &Path,
) -> Option<&'a [String]> {
    let binding = file_facts
        .module_bindings
        .as_ref()?
        .imports
        .iter()
        .filter(|import| import.specifier == source)
        .flat_map(|import| &import.bindings)
        .find(|binding| binding.local == local && binding.kind == "namespace")?;
    let members = binding.namespace_members.as_deref()?;
    receiver_safe(inputs, target, &binding.namespace_calls).then_some(members)
}

fn receiver_safe(inputs: &ExportEdgeInputs<'_>, target: &Path, calls: &[String]) -> bool {
    calls.is_empty()
        || inputs
            .facts
            .get_ts_facts(target)
            .and_then(|facts| facts.module_bindings.as_ref())
            .is_some_and(|module| {
                calls
                    .iter()
                    .all(|name| module.receiver_safe_exports.contains(name))
            })
}
