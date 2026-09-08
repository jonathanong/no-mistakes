/// Project binding-aware call facts into canonical `Call` edges.  Resolution
/// happens here, after the shared fact pass, so extractor code stays purely
/// syntactic and every graph consumer sees the same callable identity.
fn collect_call_reachability_edges(
    paths: &[PathBuf],
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    graph_files: &GraphFiles,
    interner: &PathInterner,
) -> Vec<Edge> {
    let mut edges: Vec<_> = paths
        .par_iter()
        .flat_map_iter(|path| {
            let Some(file_facts) = facts.get_ts_facts(path) else {
                return Vec::new();
            };
            file_facts
                .call_reachability
                .iter()
                .filter_map(|call| {
                    let from = call.caller.as_ref().map_or_else(
                        || NodeId::file_in(interner, path),
                        |caller| NodeId::symbol_in(interner, path, caller),
                    );
                    let to = match &call.binding {
                        crate::codebase::dependencies::extract::CallBinding::Local { scope } => {
                            NodeId::symbol_in(interner, path, scope)
                        }
                        crate::codebase::dependencies::extract::CallBinding::Import { module, export } => {
                            if let Some(visible) = resolver
                                .resolve(module, path)
                                .and_then(|resolved| graph_files.visible_path(&resolved))
                                .or_else(|| {
                                    workspace
                                        .resolve_specifier_from_file_visible(module, path, graph_files)
                                        .and_then(|resolved| graph_files.visible_path(&resolved))
                                })
                            {
                                if let Some(target) = external_star_call_target(
                                    visible,
                                    export,
                                    facts,
                                    resolver,
                                    workspace,
                                    interner,
                                ) {
                                    return Some((from, target, EdgeKind::Call));
                                }
                                if let Some((imported, member)) = export.split_once('.') {
                                    if let Some((target, _)) = resolve_reexported_namespace_member(
                                        visible,
                                        imported,
                                        member,
                                        EdgeKind::Import,
                                        ReexportNamespaceInputs {
                                            facts,
                                            resolver,
                                            workspace,
                                            visible_files: graph_files,
                                            graph_files,
                                            interner,
                                        },
                                    ) {
                                        target
                                    } else {
                                        NodeId::symbol_in(
                                            interner,
                                            visible,
                                            export.replace('.', "/"),
                                        )
                                    }
                                } else {
                                    NodeId::symbol_in(interner, visible, export)
                                }
                            } else if workspace.recognizes_specifier_from(module, path) {
                                return None;
                            } else {
                                NodeId::module_in(interner, format!("{module}#{export}"))
                            }
                        }
                        crate::codebase::dependencies::extract::CallBinding::Global { name } => {
                            NodeId::module_in(interner, format!("global:{name}"))
                        }
                        crate::codebase::dependencies::extract::CallBinding::Shadowed { .. }
                        | crate::codebase::dependencies::extract::CallBinding::Unresolved { .. } => return None,
                    };
                    Some((from, to, EdgeKind::Call))
                })
                .collect::<Vec<_>>()
        })
        .collect();
    edges.sort();
    edges.dedup();
    edges
}

/// Resolve a call through one unambiguous external `export *` source. The
/// requested call name supplies the export identity that an external package's
/// unavailable source facts cannot enumerate. Multiple star sources remain
/// unresolved because choosing one would be nondeterministic and unsound.
fn external_star_call_target(
    barrel: &Path,
    requested_export: &str,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    interner: &PathInterner,
) -> Option<NodeId> {
    let symbols = facts.get_ts_facts(barrel)?.symbols.as_ref()?;
    let requested_root = requested_export
        .split_once('.')
        .map_or(requested_export, |(root, _)| root);
    if symbols
        .exports
        .iter()
        .any(|export| export.name == requested_root)
    {
        return None;
    }
    let mut external_stars = symbols.exports.iter().filter_map(|export| {
        let ExportKind::ReExport { source, imported } = &export.kind else {
            return None;
        };
        (export.name == "*"
            && imported == "*"
            && !export.is_type_only
            && resolver.resolve(source, barrel).is_none()
            && !workspace.recognizes_specifier_from(source, barrel)
            && bare_module_node_in(interner, source).is_some())
        .then_some(source.as_str())
    });
    let source = external_stars.next()?;
    if external_stars.next().is_some() {
        return None;
    }
    Some(NodeId::module_in(
        interner,
        format!("{source}#{requested_export}"),
    ))
}
