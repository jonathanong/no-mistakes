impl SymbolIndex {
    fn build_index(
        resolver: &dyn ImportResolution,
        graph_files: &GraphFiles,
        facts: &dyn TsFactLookup,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    ) -> Self {
        let source_buckets = graph_files
            .indexable()
            .par_iter()
            .fold(
                || (SourceBuckets::new(), SymbolIndexInterner::default()),
                |(mut buckets, mut intern), path| {
                    let Some(symbols) = facts
                        .get_ts_facts(path)
                        .and_then(|facts| facts.symbols.as_ref())
                    else {
                        return (buckets, intern);
                    };

                    let expected_entries = symbols.imports.len()
                        + symbols
                            .exports
                            .iter()
                            .filter(|exp| {
                                matches!(
                                    &exp.kind,
                                    crate::codebase::ts_symbols::ExportKind::ReExport { .. }
                                )
                            })
                            .count();
                    let importer = intern.path(path);

                    for ni in &symbols.imports {
                        if let Some(target) = resolver
                            .classify_import(&ni.source, path, workspace, graph_files)
                            .preferred_path()
                            .and_then(|target| graph_files.visible_path(target))
                        {
                            insert_source_bucket_entry(
                                &mut buckets,
                                intern.path(target),
                                intern.string(&ni.imported),
                                (importer.clone(), intern.string(&ni.local), false),
                                expected_entries,
                            );
                        }
                    }
                    for exp in &symbols.exports {
                        if let crate::codebase::ts_symbols::ExportKind::ReExport {
                            source,
                            imported,
                        } = &exp.kind
                        {
                            if let Some(target) = resolver
                                .classify_import(source, path, workspace, graph_files)
                                .preferred_path()
                                .and_then(|target| graph_files.visible_path(target))
                            {
                                insert_source_bucket_entry(
                                    &mut buckets,
                                    intern.path(target),
                                    intern.string(imported),
                                    (importer.clone(), intern.string(&exp.name), true),
                                    expected_entries,
                                );
                            }
                        }
                    }
                    (buckets, intern)
                },
            )
            .reduce(
                || (SourceBuckets::new(), SymbolIndexInterner::default()),
                |(left, _), (right, _)| {
                    (
                        merge_source_buckets(left, right),
                        SymbolIndexInterner::default(),
                    )
                },
            )
            .0;

        Self::from_source_buckets(source_buckets)
    }
}
