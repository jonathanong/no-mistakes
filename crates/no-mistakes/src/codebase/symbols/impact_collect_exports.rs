fn export_paths(
    graph: &DepGraph,
    facts: &TsFactMap,
    target: &NodeId,
    target_symbol: &str,
    root: &Path,
    definition: &SymbolLocation,
) -> (Vec<SymbolLocation>, BTreeSet<NodeId>) {
    let mut exports = BTreeSet::from([definition.clone()]);
    let mut export_nodes = BTreeSet::from([target.clone()]);
    let mut frontier = vec![(target.clone(), target_symbol.to_string())];
    let mut seen = BTreeSet::from([target.clone()]);
    frontier.push((NodeId::File(root.join(&definition.file)), target_symbol.to_string()));
    while let Some((node, current_symbol)) = frontier.pop() {
        if let Some(neighbors) = graph.dependents_of_node(&node) {
            for (neighbor, _) in neighbors {
                let NodeId::Symbol { file, symbol } = neighbor else {
                    continue;
                };
                if seen.insert(neighbor.clone()) {
                    if let Some(location) = export_location(facts, file, root, symbol, true).ok().flatten() {
                        let local_import_export =
                            local_import_export(facts, file, symbol, &current_symbol);
                        if location.kind == "re-export" || local_import_export {
                            frontier.push((neighbor.clone(), symbol.clone()));
                            frontier.push((NodeId::File(file.clone()), symbol.clone()));
                            exports.insert(location);
                            export_nodes.insert(neighbor.clone());
                        }
                    }
                }
            }
        }
    }
    (exports.into_iter().collect(), export_nodes)
}

fn local_import_export(
    facts: &TsFactMap,
    file: &Path,
    symbol: &str,
    current_symbol: &str,
) -> bool {
    facts
        .get(file)
        .and_then(|facts| Some((facts.source.as_ref()?, facts.symbols.as_ref()?)))
        .and_then(|(source, symbols)| {
            let local = symbols.exports.iter().find_map(|export| {
                if matches!(export.kind, ExportKind::ReExport { .. })
                    || export_name(&export.kind, &export.name) != symbol
                {
                    return None;
                }
                Some(export.local.as_deref().unwrap_or(&export.name))
            })?;
            let direct_import_export = symbols.imports.iter().any(|import| {
                import.local == local && (import.imported == current_symbol || import.imported == "*")
            });
            (direct_import_export
                || value_alias_export(source, &symbols.imports, symbol, current_symbol))
            .then_some(())
        })
        .is_some()
}

pub(super) fn value_alias_export(
    source: &str,
    imports: &[crate::codebase::ts_symbols::NamedImport],
    export_symbol: &str,
    target_symbol: &str,
) -> bool {
    imports.iter().any(|import| {
        if import.imported == target_symbol {
            return export_alias_assigned_to(source, export_symbol, &import.local);
        }
        if import.imported == "*" {
            let member = format!("{}.{}", import.local, target_symbol);
            return export_alias_assigned_to(source, export_symbol, &member);
        }
        false
    })
}

fn export_alias_assigned_to(source: &str, export_symbol: &str, value: &str) -> bool {
    ["const", "let", "var"].iter().any(|kind| {
        let prefix = format!("export {kind} {export_symbol} =");
        source.lines().any(|line| {
            let line = line.trim();
            let Some(rhs) = line.strip_prefix(&prefix).map(str::trim) else {
                return false;
            };
            rhs.trim_end_matches(';').trim() == value
        })
    })
}
