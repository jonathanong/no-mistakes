fn project_namespace_value_exports(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    names: &FxHashMap<std::path::PathBuf, ExportNameEdges>,
    edges: &mut Vec<Edge>,
) {
    let mut pending: std::collections::BTreeSet<_> = edges
        .iter()
        .filter_map(|(_, target, kind)| {
            if *kind == EdgeKind::SymbolImport {
                if let NodeId::File(file) = target {
                    return Some(file.to_path_buf());
                }
            }
            None
        })
        .collect();
    let mut visited = fx_set();
    let projection = ValueImportProjection {
        inputs,
        facts,
        resolver,
        indexes,
    };
    while let Some(namespace) = pending.pop_first() {
        if !visited.insert(namespace.clone()) {
            continue;
        }
        for export in exported_names_for_file(&namespace, names, &mut fx_set(), true) {
            let target = match projection.resolve(&namespace, &export) {
                ExportedCallableResolution::Value(producer, local) => {
                    if !indexes
                        .file(facts, &producer)
                        .is_some_and(|file| file.values.immutable_binding(0, &local))
                        || !facts
                            .get_ts_facts(&producer)
                            .is_some_and(value_import_facts_complete)
                    {
                        continue;
                    }
                    NodeId::symbol_in(&inputs.interner, &producer, local)
                }
                ExportedCallableResolution::Namespace(target) => {
                    pending.insert(target.clone());
                    NodeId::file_in(&inputs.interner, target)
                }
                _ => continue,
            };
            edges.push((
                NodeId::file_in(&inputs.interner, &namespace),
                target,
                EdgeKind::SymbolReexport,
            ));
        }
    }
}
