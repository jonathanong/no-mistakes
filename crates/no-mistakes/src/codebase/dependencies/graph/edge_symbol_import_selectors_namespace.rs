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
    if !binding.namespace_calls.is_empty() {
        let module = inputs
            .facts
            .get_ts_facts(target)?
            .module_bindings
            .as_ref()?;
        if !binding
            .namespace_calls
            .iter()
            .all(|name| module.receiver_safe_exports.contains(name))
        {
            return None;
        }
    }
    Some(members)
}
