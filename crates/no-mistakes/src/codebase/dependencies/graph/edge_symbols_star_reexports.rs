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
    let shadowed_exports = explicit_export_keys(inputs.symbols);
    let mut visited = HashSet::new();
    collect_star_reexport_target(
        inputs,
        &target,
        &shadowed_exports,
        StarReexportKind {
            export_is_type_only: export.is_type_only,
            source_kind,
        },
        edges,
        &mut visited,
    );
}

#[derive(Clone, Copy)]
struct StarReexportKind {
    export_is_type_only: bool,
    source_kind: EdgeKind,
}

fn collect_star_reexport_target(
    inputs: &ExportEdgeInputs<'_>,
    target: &Path,
    shadowed_exports: &HashSet<StarExportKey>,
    kind: StarReexportKind,
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
                shadowed_exports,
                kind,
                edges,
                visited,
            );
        } else {
            collect_concrete_star_reexport(
                inputs,
                target,
                target_export,
                shadowed_exports,
                kind,
                edges,
            );
        }
    }
}

fn collect_concrete_star_reexport(
    inputs: &ExportEdgeInputs<'_>,
    target: &Path,
    target_export: &crate::codebase::ts_symbols::Export,
    shadowed_exports: &HashSet<StarExportKey>,
    reexport_kind: StarReexportKind,
    edges: &mut Vec<Edge>,
) {
    let reexported_symbol = export_symbol_name(target_export);
    let export_key = star_export_key(target_export, reexport_kind.export_is_type_only);
    if reexported_symbol == "default" || shadowed_exports.contains(&export_key) {
        return;
    }
    let kind = if target_export.is_type_only || reexport_kind.export_is_type_only {
        EdgeKind::TypeImport
    } else {
        reexport_kind.source_kind
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
    shadowed_exports: &HashSet<StarExportKey>,
    reexport_kind: StarReexportKind,
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
    let source_kind = if reexport_kind.source_kind == EdgeKind::TypeImport || export.is_type_only {
        EdgeKind::TypeImport
    } else {
        nested_kind
    };
    let kind = StarReexportKind {
        export_is_type_only: reexport_kind.export_is_type_only || export.is_type_only,
        source_kind,
    };
    let nested_shadowed_exports = shadowed_with_explicit_exports(shadowed_exports, inputs, target);
    collect_star_reexport_target(
        inputs,
        &nested,
        &nested_shadowed_exports,
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

fn shadowed_with_explicit_exports(
    shadowed_exports: &HashSet<StarExportKey>,
    inputs: &ExportEdgeInputs<'_>,
    target: &Path,
) -> HashSet<StarExportKey> {
    let mut shadowed_exports = shadowed_exports.clone();
    if let Some(symbols) = inputs
        .facts
        .get_ts_facts(target)
        .and_then(|facts| facts.symbols.as_ref())
    {
        shadowed_exports.extend(explicit_export_keys(symbols));
    }
    shadowed_exports
}
