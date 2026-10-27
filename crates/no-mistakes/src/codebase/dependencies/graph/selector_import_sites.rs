/// Source evidence attached to a canonical unknown selector relationship.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SelectorImportSite {
    pub(crate) line: usize,
    pub(crate) specifier: String,
    pub(crate) reason: &'static str,
}

type SelectorImportSites = FxHashMap<(PathBuf, NodeId), Vec<SelectorImportSite>>;

impl DepGraph {
    pub(crate) fn unknown_selector_sites(
        &self,
        importer: &Path,
        target: &NodeId,
    ) -> &[SelectorImportSite] {
        self.selector_import_sites
            .get(&(importer.to_path_buf(), target.clone()))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

fn push_selector_site(
    inputs: &ExportEdgeInputs<'_>,
    target: NodeId,
    site: SelectorImportSite,
    sites: &mut SelectorImportSites,
) {
    sites
        .entry((inputs.path.to_path_buf(), target))
        .or_default()
        .push(site);
}

fn relay_selector_site_index(
    inputs: &ExportEdgeInputs<'_>,
) -> FxHashMap<String, Vec<SelectorImportSite>> {
    let mut imports = fx_map();
    for import in &inputs.symbols.imports {
        imports.entry(import.local.as_str()).or_insert(import);
    }
    let mut sites: FxHashMap<String, Vec<SelectorImportSite>> = fx_map();
    for export in &inputs.symbols.exports {
        let (source, line) = if let ExportKind::ReExport { source, .. } = &export.kind {
            (source, export.line)
        } else {
            let local = export_local_name(export);
            let Some(import) = imports.get(local.as_str()) else {
                continue;
            };
            (&import.source, import.line)
        };
        sites
            .entry(export_symbol_name(export))
            .or_default()
            .push(SelectorImportSite {
                line: line.max(1) as usize,
                specifier: source.clone(),
                reason: "unresolved re-export selector",
            });
    }
    sites
}

fn collect_selector_relay_edges(
    inputs: ExportEdgeInputs<'_>,
    relay_edges: Vec<Edge>,
    exported: &FxHashSet<NodeId>,
    edges: &mut Vec<Edge>,
    sites: &mut SelectorImportSites,
) {
    let relay_sites = relay_selector_site_index(&inputs);
    for (from, to, relay_kind) in relay_edges {
        let NodeId::Symbol {
            symbol: export_name,
            ..
        } = &from
        else {
            continue;
        };
        if let NodeId::Symbol { file, symbol, .. } = &to {
            if !exported.contains(&to) {
                for site in relay_sites
                    .get(export_name.as_ref())
                    .into_iter()
                    .flatten()
                    .cloned()
                {
                    selector_unknown_symbol(&inputs, file.as_ref(), symbol, site, edges, sites);
                }
            }
            // Each relay edge already says whether its own export (or star) is
            // type-only; a type-only `export type *` must not retag value relays.
            let type_only = matches!(relay_kind, EdgeKind::TypeImport | EdgeKind::WorkspaceTypeImport)
                || inputs
                    .symbols
                    .exports
                    .iter()
                    .any(|export| export.is_type_only && export.name == **export_name);
            let kind = if type_only {
                EdgeKind::SymbolTypeReexport
            } else {
                EdgeKind::SymbolReexport
            };
            edges.push((from, to, kind));
        } else if matches!(to, NodeId::File(_)) {
            for mut site in relay_sites
                .get(export_name.as_ref())
                .into_iter()
                .flatten()
                .cloned()
            {
                site.reason = "namespace re-export selector";
                push_selector_site(&inputs, to.clone(), site, sites);
            }
            edges.push((
                NodeId::file_in(inputs.interner, inputs.path),
                to,
                EdgeKind::SymbolImportUnknown,
            ));
        }
    }
}
