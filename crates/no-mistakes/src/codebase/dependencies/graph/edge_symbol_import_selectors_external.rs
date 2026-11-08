#[derive(Clone, PartialEq, Eq)]
struct ExternalStarSymbol {
    file: PathBuf,
    name: String,
    external: bool,
    typed: bool,
    type_capable: bool,
    blocked: bool,
}

type ExternalStarExports = FxHashMap<String, ExternalStarSymbol>;

struct ExternalStarProjection<'a> {
    input: &'a GraphEdgeBuildInputs<'a>,
    facts: &'a dyn TsFactLookup,
    resolver: &'a dyn ImportResolution,
    workspace: &'a crate::codebase::workspaces::IndexedWorkspaceMap,
    session: &'a crate::codebase::analysis_session::AnalysisSession,
    sources: &'a SourceStore,
    demand: dashmap::DashMap<PathBuf, bool, rustc_hash::FxBuildHasher>,
    cache: dashmap::DashMap<
        (PathBuf, bool),
        Option<Arc<ExternalStarExports>>,
        rustc_hash::FxBuildHasher,
    >,
    declarations: crate::codebase::ts_resolver::declarations::DeclarationResolver<'a>,
}

impl<'a> ExternalStarProjection<'a> {
    fn new(
        input: &'a GraphEdgeBuildInputs<'a>,
        facts: &'a dyn TsFactLookup,
        resolver: &'a dyn ImportResolution,
        workspace: &'a crate::codebase::workspaces::IndexedWorkspaceMap,
        session: &'a crate::codebase::analysis_session::AnalysisSession,
        sources: &'a SourceStore,
    ) -> Self {
        let declarations = crate::codebase::ts_resolver::declarations::DeclarationResolver::new(
            session,
            input.root,
            resolver,
            input.tsconfig,
            input.tsconfig_catalog,
            sources,
            Some(workspace),
        );
        Self {
            input,
            facts,
            resolver,
            workspace,
            session,
            sources,
            declarations,
            cache: Default::default(),
            demand: Default::default(),
        }
    }

    fn extend_edges(&self, path: &Path, edges: &mut Vec<Edge>) {
        if !self.has_external_star(path, &mut fx_set()) {
            return;
        }
        let Some(exports) = self.exports(path, false, &mut fx_set()) else {
            return;
        };
        self.remove_blocked_edges(path, &exports, edges);
        for (name, origin) in exports
            .iter()
            .filter(|(_, origin)| origin.external && !origin.blocked)
        {
            let symbol = NodeId::symbol_in(&self.input.interner, path, name);
            let kind = if origin.typed {
                EdgeKind::SymbolTypeReexport
            } else {
                EdgeKind::SymbolReexport
            };
            edges.push((
                NodeId::file_in(&self.input.interner, path),
                symbol.clone(),
                kind,
            ));
            edges.push((
                symbol,
                NodeId::module(format!("{}#{}", origin.file.display(), origin.name)),
                kind,
            ));
        }
    }

    fn exports(
        &self,
        path: &Path,
        external: bool,
        active: &mut FxHashSet<PathBuf>,
    ) -> Option<Arc<ExternalStarExports>> {
        if external && self.input.graph_files.visible_path(path).is_some() {
            return None;
        }
        let key = (path.to_path_buf(), external);
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }
        if !active.insert(path.to_path_buf()) {
            return None;
        }
        let result = self.exports_inner(path, external, active).map(Arc::new);
        active.remove(path);
        self.cache.insert(key, result.clone());
        result
    }

    fn exports_inner(
        &self,
        path: &Path,
        external: bool,
        active: &mut FxHashSet<PathBuf>,
    ) -> Option<ExternalStarExports> {
        let supplemental;
        let facts = if external {
            supplemental = self.session.declaration_facts(self.input.root, path);
            supplemental.as_ref()
        } else {
            self.facts.get_ts_facts(path)?
        };
        if facts.parse_error.is_some() || facts.operational_error.is_some() {
            return None;
        }
        let symbols = facts.symbols.as_ref()?;
        let imported: FxHashSet<_> = symbols
            .imports
            .iter()
            .filter(|_| external)
            .map(|import| import.local.as_str())
            .collect();
        let mut exports = fx_map();
        for export in symbols.exports.iter().filter(|export| export.name != "*") {
            if external {
                if let ExportKind::ReExport { source, imported } = &export.kind {
                    let origin =
                        self.named_reexport(path, source, imported, export.is_type_only, active)?;
                    exports.insert(export.name.clone(), origin);
                    continue;
                }
                if imported.contains(export.local.as_deref().unwrap_or(&export.name)) {
                    return None;
                }
            }
            exports.insert(
                export.name.clone(),
                ExternalStarSymbol {
                    file: path.to_path_buf(),
                    name: export.name.clone(),
                    external,
                    blocked: false,
                    typed: export.is_type_only,
                    type_capable: export.is_type_only
                        || matches!(
                            export.kind,
                            ExportKind::Class
                                | ExportKind::Enum
                                | ExportKind::Interface
                                | ExportKind::TypeAlias
                        ),
                },
            );
        }
        let explicit: FxHashSet<_> = exports.keys().cloned().collect();
        for export in &symbols.exports {
            let ExportKind::ReExport { source, imported } = &export.kind else {
                continue;
            };
            if export.name != "*" || imported != "*" {
                continue;
            }
            let (target, is_external) = self.target(path, source, external)?;
            let nested = self.exports(&target, is_external, active)?;
            for (name, origin) in nested.iter() {
                let (name, mut origin) = (name.clone(), origin.clone());
                if name == "default" || explicit.contains(&name) {
                    continue;
                }
                if export.is_type_only && !origin.type_capable {
                    continue;
                }
                origin.typed |= export.is_type_only;
                merge_external_star_origin(&mut exports, name, origin);
            }
        }
        Some(exports)
    }
}
