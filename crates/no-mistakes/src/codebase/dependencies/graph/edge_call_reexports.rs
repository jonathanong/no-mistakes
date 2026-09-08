/// Retype only proven export-forwarding symbol edges for the call projection.
/// Ordinary imported value references retain `Import` alone and therefore
/// cannot widen call-specific traversal.
fn collect_call_reexport_edges(
    paths: &[PathBuf],
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    graph_files: &GraphFiles,
    forward: &EdgeMap,
    interner: &PathInterner,
) -> Vec<Edge> {
    let mut edges = Vec::new();
    for path in paths {
        let Some(symbols) = facts.get_ts_facts(path).and_then(|facts| facts.symbols.as_deref()) else {
            continue;
        };
        for export in &symbols.exports {
            let ExportKind::ReExport { source, imported } = &export.kind else {
                continue;
            };
            if export.name == "*" {
                continue;
            }
            let from = NodeId::symbol_in(interner, path, export_symbol_name(export));
            if resolver.resolve(source, path).is_none()
                && !workspace.recognizes_specifier_from(source, path)
                && imported != "*"
                && bare_module_node_in(interner, source).is_some()
            {
                edges.push((
                    from,
                    NodeId::module_in(interner, format!("{source}#{imported}")),
                    EdgeKind::CallReexport,
                ));
                continue;
            }
            let Some(neighbors) = forward.get(&from) else {
                continue;
            };
            edges.extend(
                neighbors
                    .iter()
                    .filter(|(_, kind)| {
                        matches!(kind, EdgeKind::Import | EdgeKind::WorkspaceImport)
                    })
                    .map(|(target, _)| {
                        (from.clone(), target.clone(), EdgeKind::CallReexport)
                    }),
            );
        }

        // Star re-exports synthesize named barrel symbols, so there is no
        // explicit `Export` record whose name can be looked up above. Reuse
        // the canonical star-export projection to preserve its ambiguity,
        // shadowing, type-only, and nested-barrel semantics exactly.
        let inputs = ExportEdgeInputs {
            path,
            symbols,
            facts,
            resolver,
            workspace,
            visible_files: graph_files,
            graph_files,
            interner,
        };
        let mut star_edges = Vec::new();
        collect_star_reexport_edges(&inputs, &mut star_edges);
        edges.extend(star_edges.into_iter().filter_map(|(from, target, kind)| {
            matches!(kind, EdgeKind::Import | EdgeKind::WorkspaceImport)
                .then_some((from, target, EdgeKind::CallReexport))
        }));
    }
    edges.sort();
    edges.dedup();

    // A re-export becomes part of the call projection only when an actual
    // resolved call reaches it (possibly through another re-export). This
    // prevents reverse call queries for ordinary exported values from gaining
    // fabricated callers while still supporting arbitrarily deep barrels.
    let mut reachable: HashSet<NodeId> = forward
        .values()
        .flat_map(|neighbors| neighbors.iter())
        .filter(|(_, kind)| *kind == EdgeKind::Call)
        .map(|(target, _)| target.clone())
        .collect();
    let mut selected = Vec::new();
    let mut pending = edges;
    loop {
        let mut progressed = false;
        pending.retain(|edge| {
            if reachable.contains(&edge.0) {
                reachable.insert(edge.1.clone());
                selected.push(edge.clone());
                progressed = true;
                false
            } else {
                true
            }
        });
        if !progressed {
            break;
        }
    }
    selected.sort();
    selected
}

struct CallReexportMergeInputs<'a> {
    plan: GraphBuildPlan,
    files: &'a [PathBuf],
    facts: Option<&'a dyn TsFactLookup>,
    resolver: &'a dyn ImportResolution,
    workspace: &'a crate::codebase::workspaces::IndexedWorkspaceMap,
    graph_files: &'a GraphFiles,
    interner: &'a PathInterner,
}

fn merge_call_reexports_if_requested(
    inputs: CallReexportMergeInputs<'_>,
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
) {
    if !inputs.plan.calls {
        return;
    }
    let edges = collect_call_reexport_edges(
        inputs.files,
        inputs
            .facts
            .expect("call plans require prepared TS facts"),
        inputs.resolver,
        inputs.workspace,
        inputs.graph_files,
        forward,
        inputs.interner,
    );
    merge_edges(forward, reverse, edges);
}
