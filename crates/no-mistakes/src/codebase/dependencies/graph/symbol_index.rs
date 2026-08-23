/// Public importer row. Interned as `Arc<Path>` / `Arc<str>` so clones stay
/// pointer-sized; convert to `PathBuf` / `String` at output boundaries.
pub type ImporterRecord = (Arc<Path>, Arc<str>, bool);

/// Index mapping (source_file, exported_symbol) → list of files importing that symbol.
pub struct SymbolIndex {
    sources: HashMap<Arc<Path>, SourceIndex>,
}

struct SourceIndex {
    by_symbol: HashMap<Arc<str>, Vec<ImporterRecord>>,
    file_importers: Vec<Arc<Path>>,
}

impl SymbolIndex {
    pub fn build(symbols_by_file: &HashMap<PathBuf, Vec<(PathBuf, String, String, bool)>>) -> Self {
        let mut intern = SymbolIndexInterner::default();
        let mut source_buckets = SourceBuckets::with_capacity(symbols_by_file.len());

        for (importer, imports) in symbols_by_file {
            let importer = intern.path(importer);
            for (source, imported_name, local_name, is_reexport) in imports {
                insert_source_bucket_entry(
                    &mut source_buckets,
                    intern.path(source),
                    intern.string(imported_name),
                    (importer.clone(), intern.string(local_name), *is_reexport),
                    imports.len(),
                );
            }
        }

        Self::from_source_buckets(source_buckets)
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
        let session =
            crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
        Self::build_from_facts_with_session(root, tsconfig, graph_files, facts, &session)
    }

    pub(crate) fn build_from_facts_workspace_resolution_cache_and_session(
        tsconfig: &TsConfig,
        tsconfig_catalog: Option<&crate::codebase::ts_resolver::TsConfigCatalog>,
        graph_files: &GraphFiles,
        facts: &dyn TsFactLookup,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        import_resolution_cache: Option<&crate::codebase::ts_resolver::ImportResolutionCache>,
        session: &crate::codebase::analysis_session::AnalysisSession,
    ) -> Self {
        let resolver = crate::codebase::ts_resolver::ProjectImportResolver::new(
            tsconfig,
            tsconfig_catalog,
            graph_files,
            import_resolution_cache,
            session,
        );
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
            None,
            graph_files,
            facts,
            &workspace,
            None,
            session,
        )
    }

    fn from_source_buckets(source_buckets: SourceBuckets) -> Self {
        let mut sources = HashMap::with_capacity(source_buckets.len());

        for (source, entries) in source_buckets {
            let mut importers = Vec::with_capacity(entries.len());
            let mut by_symbol = HashMap::with_capacity(entries.len());
            for (imported_name, importer) in entries {
                importers.push(importer.0.clone());
                by_symbol
                    .entry(imported_name)
                    .or_insert_with(Vec::new)
                    .push(importer);
            }
            importers.sort_by(|left, right| left.as_os_str().cmp(right.as_os_str()));
            importers.dedup();
            sources.insert(
                source,
                SourceIndex {
                    by_symbol,
                    file_importers: importers,
                },
            );
        }

        Self { sources }
    }

    pub fn importers_of(&self, source: &Path, symbol: &str) -> Option<&Vec<ImporterRecord>> {
        self.sources.get(source)?.by_symbol.get(symbol)
    }

    /// Files that import any exported symbol from `source`, regardless of which
    /// symbol. Deduplicated and sorted. Powers file-level reverse queries
    /// (`importers`, `exports-of`) without building the full dependency graph.
    pub fn file_importers(&self, source: &Path) -> Vec<PathBuf> {
        self.sources
            .get(source)
            .map(|index| {
                index
                    .file_importers
                    .iter()
                    .map(|path| path.to_path_buf())
                    .collect()
            })
            .unwrap_or_default()
    }
}
