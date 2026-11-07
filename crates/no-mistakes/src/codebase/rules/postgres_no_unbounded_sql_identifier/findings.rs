use super::*;

pub(super) fn identifier_finding(
    root: &Path,
    graph: &DepGraph,
    facts: &CheckFactMap,
    routes: &RegistryRoutes<'_>,
    call: &SqlCall<'_>,
    provenance: &Provenance,
) -> Option<RuleFinding> {
    match provenance {
        Provenance::ProcessEnv(key) => {
            if shadowed_process(facts, &call.file) {
                return None;
            }
            Some(message(
                root,
                call,
                &format!("process.env.{key}"),
                Some(format!("process.env.{key}")),
            ))
        }
        Provenance::Reference { name, scope, .. } => {
            named_finding(root, graph, facts, routes, call, (*scope, name))
        }
        Provenance::Namespace {
            local, exported, ..
        } => {
            let unique = unique_namespace_member(graph, facts, &call.file, local, exported)?;
            imported_unbounded(root, facts, call, unique.producer, &unique.exported, local)
        }
    }
}

fn named_finding(
    root: &Path,
    graph: &DepGraph,
    facts: &CheckFactMap,
    routes: &RegistryRoutes<'_>,
    call: &SqlCall<'_>,
    (scope, name): (usize, &str),
) -> Option<RuleFinding> {
    if local_env_alias(facts, &call.file, name) || parameter(facts, &call.file, scope, name) {
        return Some(message(root, call, name, Some(name.to_string())));
    }
    if query_result_binding(facts, &call.file, name) {
        return Some(message(root, call, name, Some(name.to_string())));
    }
    let unique = unique_export(graph, facts, routes, &call.file, name)?;
    if unique.producer == call.file {
        return None;
    }
    imported_unbounded(root, facts, call, unique.producer, &unique.exported, name)
}

pub(super) fn message(
    root: &Path,
    call: &SqlCall<'_>,
    producer: &str,
    import: Option<String>,
) -> RuleFinding {
    finding(
        root,
        RULE_ID,
        &call.file,
        call.call.line,
        diagnostic_message(producer),
        import,
        Some(producer.to_string()),
    )
}

pub(in super::super) fn diagnostic_message(producer: &str) -> String {
    format!("Unbounded identifier source `{producer}` reaches SQL relation interpolation in this executor call without a runtime finite-domain validator. Map accepted values to literal schemas with a switch whose default throws; a type assertion is not validation. Bound values remain allowed.")
}
