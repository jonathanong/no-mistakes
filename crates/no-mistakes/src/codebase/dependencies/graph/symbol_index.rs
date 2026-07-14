pub type ImporterRecord = (PathBuf, String, bool);

/// Index mapping (source_file, exported_symbol) → list of files importing that symbol.
pub struct SymbolIndex {
    map: HashMap<(PathBuf, String), Vec<ImporterRecord>>,
}

impl SymbolIndex {
    pub fn build(symbols_by_file: &HashMap<PathBuf, Vec<(PathBuf, String, String, bool)>>) -> Self {
        let mut map: HashMap<(PathBuf, String), Vec<ImporterRecord>> = HashMap::new();

        for (importer, imports) in symbols_by_file {
            for (source, imported_name, local_name, is_reexport) in imports {
                map.entry((source.clone(), imported_name.clone()))
                    .or_default()
                    .push((importer.clone(), local_name.clone(), *is_reexport));
            }
        }

        Self { map }
    }

    /// Build a symbol import index for every indexable file under `root`.
    ///
    /// This is the companion index required by `DepGraph::dependents_of_symbol`
    /// for `file#exportName` queries.
    pub fn build_from_root(root: &Path, tsconfig: &TsConfig) -> Result<Self> {
        let graph_files = GraphFiles::discover(root);
        Ok(Self::build_from_files(root, tsconfig, &graph_files))
    }

    pub(crate) fn build_from_files(
        root: &Path,
        tsconfig: &TsConfig,
        graph_files: &GraphFiles,
    ) -> Self {
        let facts = collect_ts_facts(graph_files.indexable(), TsFactPlan::imports_and_symbols());
        Self::build_from_facts(root, tsconfig, graph_files, &facts)
    }

    pub(crate) fn build_from_facts(
        root: &Path,
        tsconfig: &TsConfig,
        graph_files: &GraphFiles,
        facts: &TsFactMap,
    ) -> Self {
        let session = crate::codebase::analysis_session::AnalysisSession::new(
            crate::diagnostics::current(),
        );
        Self::build_from_facts_with_session(root, tsconfig, graph_files, facts, &session)
    }

    pub(crate) fn build_from_facts_and_workspace(
        tsconfig: &TsConfig,
        graph_files: &GraphFiles,
        facts: &dyn TsFactLookup,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    ) -> Self {
        let session = crate::codebase::analysis_session::AnalysisSession::new(
            crate::diagnostics::current(),
        );
        Self::build_from_facts_workspace_resolution_cache_and_session(
            tsconfig,
            graph_files,
            facts,
            workspace,
            None,
            &session,
        )
    }

    pub(crate) fn build_from_facts_workspace_and_resolution_cache(
        tsconfig: &TsConfig,
        graph_files: &GraphFiles,
        facts: &dyn TsFactLookup,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        import_resolution_cache: Option<&crate::codebase::ts_resolver::ImportResolutionCache>,
    ) -> Self {
        let session = crate::codebase::analysis_session::AnalysisSession::new(
            crate::diagnostics::current(),
        );
        Self::build_from_facts_workspace_resolution_cache_and_session(
            tsconfig,
            graph_files,
            facts,
            workspace,
            import_resolution_cache,
            &session,
        )
    }

    pub(crate) fn build_from_facts_workspace_resolution_cache_and_session(
        tsconfig: &TsConfig,
        graph_files: &GraphFiles,
        facts: &dyn TsFactLookup,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        import_resolution_cache: Option<&crate::codebase::ts_resolver::ImportResolutionCache>,
        session: &crate::codebase::analysis_session::AnalysisSession,
    ) -> Self {
        let resolver = ImportResolver::new_observed(tsconfig, session.observer().cloned())
            .with_visible(graph_files.visible());
        let resolver = match import_resolution_cache {
            Some(cache) => resolver.with_shared_cache(cache),
            None => resolver,
        };
        session.record_work("symbol_index.builds", 1);
        Self::build_index(&resolver, graph_files, facts, workspace)
    }

    pub(crate) fn build_from_facts_with_session(
        root: &Path,
        tsconfig: &TsConfig,
        graph_files: &GraphFiles,
        facts: &TsFactMap,
        session: &crate::codebase::analysis_session::AnalysisSession,
    ) -> Self {
        let dataset = crate::codebase::analysis_dataset::AnalysisDataset::new_observed(
            root,
            session.observer().cloned(),
        );
        let workspace = dataset.workspace();
        Self::build_from_facts_workspace_resolution_cache_and_session(
            tsconfig,
            graph_files,
            facts,
            &workspace,
            None,
            session,
        )
    }

    fn build_index(
        resolver: &ImportResolver<'_>,
        graph_files: &GraphFiles,
        facts: &dyn TsFactLookup,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    ) -> Self {
        type SymEntry = (PathBuf, String, String, bool);

        let per_file: Vec<(PathBuf, Vec<SymEntry>)> = graph_files
            .indexable()
            .par_iter()
            .filter_map(|path| {
                let symbols = facts.get_ts_facts(path)?.symbols.as_ref()?;

                let mut imports_for_file = Vec::new();
                for ni in &symbols.imports {
                    if let Some(target) = resolver
                        .classify_import(&ni.source, path, workspace, graph_files.visible())
                        .preferred_path() {
                        imports_for_file.push((
                            target.to_path_buf(),
                            ni.imported.clone(),
                            ni.local.clone(),
                            false,
                        ));
                    }
                }
                for exp in &symbols.exports {
                    if let crate::codebase::ts_symbols::ExportKind::ReExport { source, imported } =
                        &exp.kind
                    {
                        if let Some(target) = resolver
                            .classify_import(source, path, workspace, graph_files.visible())
                            .preferred_path() {
                            imports_for_file.push((
                                target.to_path_buf(),
                                imported.clone(),
                                exp.name.clone(),
                                true,
                            ));
                        }
                    }
                }

                if imports_for_file.is_empty() {
                    None
                } else {
                    Some((path.clone(), imports_for_file))
                }
            })
            .collect();

        let symbols_by_file: HashMap<PathBuf, Vec<SymEntry>> = per_file.into_iter().collect();

        Self::build(&symbols_by_file)
    }

    pub fn importers_of(&self, source: &Path, symbol: &str) -> Option<&Vec<ImporterRecord>> {
        self.map.get(&(source.to_path_buf(), symbol.to_string()))
    }

    /// Files that import any exported symbol from `source`, regardless of which
    /// symbol. Deduplicated and sorted. Powers file-level reverse queries
    /// (`importers`, `exports-of`) without building the full dependency graph.
    pub fn file_importers(&self, source: &Path) -> Vec<PathBuf> {
        let source = source.to_path_buf();
        let mut importers: Vec<PathBuf> = self
            .map
            .iter()
            .filter(|((file, _), _)| file == &source)
            .flat_map(|(_, records)| records.iter().map(|(importer, _, _)| importer.clone()))
            .collect();
        importers.sort();
        importers.dedup();
        importers
    }
}
