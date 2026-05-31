fn resolve_reexported_namespace_member(
    barrel: &Path,
    imported: &str,
    member: &str,
    kind: EdgeKind,
    facts: &dyn TsFactLookup,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
) -> Option<(NodeId, EdgeKind)> {
    ReexportNamespaceResolver {
        member,
        facts,
        resolver,
        workspace,
        visited: HashSet::new(),
    }
    .resolve(barrel, imported, kind)
}

struct ReexportNamespaceResolver<'a, 'b> {
    member: &'a str,
    facts: &'a dyn TsFactLookup,
    resolver: &'a ImportResolver<'b>,
    workspace: &'a crate::codebase::workspaces::WorkspaceMap,
    visited: HashSet<(PathBuf, String)>,
}

impl ReexportNamespaceResolver<'_, '_> {
    fn resolve(
        &mut self,
        barrel: &Path,
        imported: &str,
        kind: EdgeKind,
    ) -> Option<(NodeId, EdgeKind)> {
        if !self
            .visited
            .insert((barrel.to_path_buf(), imported.to_string()))
        {
            return None;
        }
        let barrel_symbols = self.facts.get_ts_facts(barrel)?.symbols.as_ref()?;
        for export in &barrel_symbols.exports {
            if export.name != imported {
                continue;
            }
            let ExportKind::ReExport {
                source,
                imported: reexported,
            } = &export.kind
            else {
                let local = export_local_name(export);
                let namespace_imports =
                    namespace_import_map(barrel, barrel_symbols, self.resolver, self.workspace);
                if let Some(target) = namespace_imports.get(&local) {
                    return Some(namespace_target_node(target, self.member));
                }
                continue;
            };
            let (target, source_kind) = if let Some(target) = self.resolver.resolve(source, barrel) {
                (target, kind)
            } else {
                (
                    self.workspace.resolve_specifier_from(source, barrel)?,
                    EdgeKind::WorkspaceImport,
                )
            };
            let edge_kind = if kind == EdgeKind::TypeImport || export.is_type_only {
                EdgeKind::TypeImport
            } else {
                source_kind
            };
            if reexported == "*" {
                return Some((
                    NodeId::Symbol {
                        file: target,
                        symbol: self.member.to_string(),
                    },
                    edge_kind,
                ));
            }
            if let Some(resolved) = self.resolve(&target, reexported, edge_kind) {
                return Some(resolved);
            }
        }
        None
    }
}
