fn collect_star_reexport_edges(
    inputs: &ExportEdgeInputs<'_>,
    export: &crate::codebase::ts_symbols::Export,
    edges: &mut Vec<Edge>,
) {
    let ExportKind::ReExport { source, imported } = &export.kind else {
        return;
    };
    if imported != "*" {
        return;
    }
    let target = resolve_star_source(inputs, inputs.path, source, export.is_type_only);
    let Some((target, source_kind)) = target else {
        return;
    };
    let mut visited = HashSet::new();
    collect_star_reexport_target(
        inputs,
        &target,
        export.is_type_only,
        source_kind,
        edges,
        &mut visited,
    );
}

fn collect_star_reexport_target(
    inputs: &ExportEdgeInputs<'_>,
    target: &Path,
    export_is_type_only: bool,
    source_kind: EdgeKind,
    edges: &mut Vec<Edge>,
    visited: &mut HashSet<PathBuf>,
) {
    if !visited.insert(target.to_path_buf()) {
        return;
    }
    let Some(target_symbols) = inputs
        .facts
        .get_ts_facts(target)
        .and_then(|facts| facts.symbols.as_ref())
    else {
        return;
    };
    for target_export in &target_symbols.exports {
        if target_export.name == "*" {
            collect_nested_star_reexport(
                inputs,
                target,
                target_export,
                export_is_type_only,
                source_kind,
                edges,
                visited,
            );
        } else {
            collect_concrete_star_reexport(inputs, target, target_export, export_is_type_only, source_kind, edges);
        }
    }
}

fn collect_concrete_star_reexport(
    inputs: &ExportEdgeInputs<'_>,
    target: &Path,
    target_export: &crate::codebase::ts_symbols::Export,
    export_is_type_only: bool,
    source_kind: EdgeKind,
    edges: &mut Vec<Edge>,
) {
    let reexported_symbol = export_symbol_name(target_export);
    if reexported_symbol == "default" {
        return;
    }
    let kind = if target_export.is_type_only || export_is_type_only {
        EdgeKind::TypeImport
    } else {
        source_kind
    };
    edges.push((
        NodeId::File(inputs.path.to_path_buf()),
        NodeId::Symbol {
            file: inputs.path.to_path_buf(),
            symbol: reexported_symbol.clone(),
        },
        kind,
    ));
    edges.push((
        NodeId::Symbol {
            file: inputs.path.to_path_buf(),
            symbol: reexported_symbol.clone(),
        },
        NodeId::Symbol {
            file: target.to_path_buf(),
            symbol: reexported_symbol,
        },
        kind,
    ));
}

fn collect_nested_star_reexport(
    inputs: &ExportEdgeInputs<'_>,
    target: &Path,
    export: &crate::codebase::ts_symbols::Export,
    export_is_type_only: bool,
    source_kind: EdgeKind,
    edges: &mut Vec<Edge>,
    visited: &mut HashSet<PathBuf>,
) {
    let ExportKind::ReExport { source, imported } = &export.kind else {
        return;
    };
    if imported != "*" {
        return;
    }
    let nested = resolve_star_source(inputs, target, source, export.is_type_only);
    let Some((nested, nested_kind)) = nested else {
        return;
    };
    let kind = if source_kind == EdgeKind::TypeImport || export.is_type_only {
        EdgeKind::TypeImport
    } else {
        nested_kind
    };
    collect_star_reexport_target(
        inputs,
        &nested,
        export_is_type_only || export.is_type_only,
        kind,
        edges,
        visited,
    );
}

fn resolve_star_source(
    inputs: &ExportEdgeInputs<'_>,
    from: &Path,
    source: &str,
    is_type_only: bool,
) -> Option<(PathBuf, EdgeKind)> {
    if let Some(target) = inputs.resolver.resolve(source, from) {
        Some((target, symbol_edge_kind(is_type_only)))
    } else {
        inputs
            .workspace
            .resolve_specifier_from(source, from)
            .map(|target| (target, EdgeKind::WorkspaceImport))
    }
}
