pub type ImporterRecord = (PathBuf, String, bool);

/// Index mapping (source_file, exported_symbol) → list of files importing that symbol.
pub struct SymbolIndex {
    sources: HashMap<PathBuf, SourceIndex>,
}

struct SourceIndex {
    by_symbol: HashMap<String, Vec<ImporterRecord>>,
    file_importers: Vec<PathBuf>,
}

impl SymbolIndex {
    pub fn build(symbols_by_file: &HashMap<PathBuf, Vec<(PathBuf, String, String, bool)>>) -> Self {
        let mut source_buckets = SourceBuckets::new();

        for (importer, imports) in symbols_by_file {
            for (source, imported_name, local_name, is_reexport) in imports {
                source_buckets.entry(source.clone()).or_default().push((
                    imported_name.clone(),
                    (importer.clone(), local_name.clone(), *is_reexport),
                ));
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
            graph_files.visible(),
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

    fn build_index(
        resolver: &dyn ImportResolution,
        graph_files: &GraphFiles,
        facts: &dyn TsFactLookup,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    ) -> Self {
        let source_buckets = graph_files
            .indexable()
            .par_iter()
            .fold(SourceBuckets::new, |mut buckets, path| {
                let Some(symbols) = facts
                    .get_ts_facts(path)
                    .and_then(|facts| facts.symbols.as_ref())
                else {
                    return buckets;
                };

                for ni in &symbols.imports {
                    if let Some(target) = resolver
                        .classify_import(&ni.source, path, workspace, graph_files.visible())
                        .preferred_path()
                        .and_then(|target| graph_files.visible_path(target))
                    {
                        buckets
                            .entry(target.to_path_buf())
                            .or_default()
                            .push((ni.imported.clone(), (path.clone(), ni.local.clone(), false)));
                    }
                }
                for exp in &symbols.exports {
                    if let crate::codebase::ts_symbols::ExportKind::ReExport { source, imported } =
                        &exp.kind
                    {
                        if let Some(target) = resolver
                            .classify_import(source, path, workspace, graph_files.visible())
                            .preferred_path()
                            .and_then(|target| graph_files.visible_path(target))
                        {
                            buckets
                                .entry(target.to_path_buf())
                                .or_default()
                                .push((imported.clone(), (path.clone(), exp.name.clone(), true)));
                        }
                    }
                }
                buckets
            })
            .reduce(SourceBuckets::new, merge_source_buckets);

        Self::from_source_buckets(source_buckets)
    }

    fn from_source_buckets(source_buckets: SourceBuckets) -> Self {
        let mut sources = HashMap::with_capacity(source_buckets.len());

        for (source, entries) in source_buckets {
            let mut importers = Vec::with_capacity(entries.len());
            let mut by_symbol = HashMap::new();
            for (imported_name, importer) in entries {
                importers.push(importer.0.clone());
                by_symbol
                    .entry(imported_name)
                    .or_insert_with(Vec::new)
                    .push(importer);
            }
            importers.sort();
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
            .map(|index| index.file_importers.clone())
            .unwrap_or_default()
    }
}

type SourceBucketEntry = (String, ImporterRecord);
type SourceBuckets = HashMap<PathBuf, Vec<SourceBucketEntry>>;

fn merge_source_buckets(mut left: SourceBuckets, right: SourceBuckets) -> SourceBuckets {
    for (source, mut entries) in right {
        left.entry(source).or_default().append(&mut entries);
    }
    left
}
