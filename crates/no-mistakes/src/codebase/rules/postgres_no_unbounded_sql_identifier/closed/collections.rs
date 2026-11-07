use super::*;

pub(super) fn proven(
    graph: &DepGraph,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    file: &Path,
    name: &str,
    kind: RegistryKind,
) -> bool {
    let Some(ts) = facts.get_ts_facts(file) else {
        return false;
    };
    let flow = &ts.value_flow;
    if unique_closed_collection(flow, name, kind) {
        return true;
    }
    if flow.bindings.iter().any(|binding| binding.name == name)
        || flow
            .parameters
            .iter()
            .any(|parameter| parameter.name == name)
        || flow
            .registry_mutations
            .iter()
            .any(|mutation| mutation.name == name)
    {
        return false;
    }
    let Some(export) = unique_export(graph, facts, &bindings.routes, file, name) else {
        return false;
    };
    let Some(producer) = facts.get_ts_facts(&export.producer) else {
        return false;
    };
    let flow = &producer.value_flow;
    let closed = flow.closed_string_registries.iter().any(|registry| {
        registry.scope == 0 && registry.name == export.exported && registry.kind == kind
    });
    closed
        && bindings.imported_collection_safe(&export.producer, &export.exported, || {
            super::collection_imports::safe(graph, facts, bindings, &export)
        })
}
