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
        Ok(Self::build_from_files(tsconfig, &graph_files))
    }

    pub(crate) fn build_from_files(tsconfig: &TsConfig, graph_files: &GraphFiles) -> Self {
        let facts = collect_ts_facts(graph_files.indexable(), TsFactPlan::imports_and_symbols());
        Self::build_from_facts(tsconfig, graph_files, &facts)
    }

    pub(crate) fn build_from_facts(
        tsconfig: &TsConfig,
        graph_files: &GraphFiles,
        facts: &TsFactMap,
    ) -> Self {
        type SymEntry = (PathBuf, String, String, bool);

        let resolver = ImportResolver::new(tsconfig).with_visible(graph_files.visible());

        let per_file: Vec<(PathBuf, Vec<SymEntry>)> = graph_files
            .indexable()
            .par_iter()
            .filter_map(|path| {
                let symbols = facts.get(path)?.symbols.as_ref()?;

                let mut imports_for_file = Vec::new();
                for ni in &symbols.imports {
                    if let Some(target) = resolver.resolve(&ni.source, path) {
                        imports_for_file.push((
                            target,
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
                        if let Some(target) = resolver.resolve(source, path) {
                            imports_for_file.push((
                                target,
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
}

