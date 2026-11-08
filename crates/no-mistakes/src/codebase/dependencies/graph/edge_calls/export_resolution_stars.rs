fn resolve_star_export(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    selection: (&std::path::Path, &str, &CallableFileIndex),
    indexes: &CallableResolutionIndexes,
    visited: &mut ExportResolutionTrace,
) -> ExportedCallableResolution {
    let (path, export, file) = selection;
    // A callable's spelling alone is not an export. Keeping this after the
    // explicit binding lookup prevents a private `fn sameName()` in a barrel
    // from satisfying an imported selector. ECMAScript `export *` deliberately
    // excludes `default`; guessing one from a barrel is incorrect.
    if export == "default" {
        ExportedCallableResolution::Absent
    } else {
        let mut candidates = Vec::new();
        let mut values = Vec::new();
        let mut namespaces = Vec::new();
        let mut external_namespaces = Vec::new();
        let mut external = Vec::new();
        let mut has_unknown_candidate = false;
        for specifier in &file.stars {
            // An unresolved or excluded star source can still export this
            // name. It therefore collides with any callable branch just as
            // a visible non-callable source does; treating it as absent
            // would manufacture an unsound call edge.
            let Some(target_path) = resolver.resolve(specifier, path) else {
                has_unknown_candidate = true;
                continue;
            };
            let Some(target_path) = edge_inputs.graph_files.visible_path(&target_path) else {
                has_unknown_candidate = true;
                continue;
            };
            let mut branch_visited = visited.clone();
            match resolve_exported_callable(
                edge_inputs,
                facts,
                resolver,
                target_path,
                export,
                indexes,
                &mut branch_visited,
            ) {
                ExportedCallableResolution::Absent => {}
                ExportedCallableResolution::Callable(path, scope, callable_id) => {
                    candidates.push((path, scope, callable_id))
                }
                ExportedCallableResolution::Value(path, local) => values.push((path, local)),
                ExportedCallableResolution::Namespace(path) => namespaces.push(path),
                ExportedCallableResolution::ExternalNamespace(module) => external_namespaces.push(module),
                ExportedCallableResolution::ExternalModuleExport(module, name) => {
                    external.push((module, name))
                }
                ExportedCallableResolution::Unknown => has_unknown_candidate = true,
            }
            visited.failures.extend(branch_visited.failures);
        }
        candidates.sort();
        candidates.dedup();
        values.sort();
        values.dedup();
        external.sort();
        external.dedup();
        namespaces.sort();
        namespaces.dedup();
        external_namespaces.sort();
        external_namespaces.dedup();
        if has_unknown_candidate
            || candidates.len() + values.len() + external.len() + namespaces.len() + external_namespaces.len() > 1
        {
            ExportedCallableResolution::Unknown
        } else if let Some(module) = external_namespaces.pop() {
            ExportedCallableResolution::ExternalNamespace(module)
        } else if let Some((module, name)) = external.pop() {
            ExportedCallableResolution::ExternalModuleExport(module, name)
        } else if let Some((path, local)) = values.pop() {
            ExportedCallableResolution::Value(path, local)
        } else if let Some(path) = namespaces.pop() {
            ExportedCallableResolution::Namespace(path)
        } else if let Some((path, scope, callable_id)) = candidates.pop() {
            ExportedCallableResolution::Callable(path, scope, callable_id)
        } else {
            ExportedCallableResolution::Absent
        }
    }
}
