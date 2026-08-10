pub(crate) fn route_defs_from_files_with_catalog(
    root: &Path,
    files: &[PathBuf],
    tsconfig: &TsConfig,
    tsconfig_catalog: Option<&TsConfigCatalog>,
) -> Vec<(PathBuf, String)> {
    let root = root.canonicalize().unwrap_or(root.to_path_buf());
    let facts = collect_file_facts(files, &root);
    build_route_defs(&root, &facts, tsconfig, tsconfig_catalog)
}

pub(crate) fn route_defs_from_prepared_facts_with_catalog(
    root: &Path,
    tsconfig: &TsConfig,
    tsconfig_catalog: Option<&TsConfigCatalog>,
    prepared: impl IntoIterator<Item = (PathBuf, FileFacts)>,
) -> Vec<(PathBuf, String)> {
    let root = root.canonicalize().unwrap_or(root.to_path_buf());
    let facts = prepared.into_iter().collect();
    build_route_defs(&root, &facts, tsconfig, tsconfig_catalog)
}

fn build_route_defs(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    tsconfig: &TsConfig,
    tsconfig_catalog: Option<&TsConfigCatalog>,
) -> Vec<(PathBuf, String)> {
    let visible = facts.keys().cloned().collect::<HashSet<_>>();
    let report = if let Some(catalog) = tsconfig_catalog {
        let resolver = ScopedImportResolver::from_visible(catalog, &visible);
        let session = crate::codebase::analysis_session::AnalysisSession::disabled();
        build_report_with_resolver(root, facts, &Default::default(), tsconfig, &session, &resolver)
    } else {
        build_report(root, facts, tsconfig)
    };
    report
        .routes
        .into_iter()
        .map(|route| (root.join(route.file), route.route))
        .collect()
}

fn collect_file_facts(files: &[PathBuf], root: &Path) -> HashMap<PathBuf, FileFacts> {
    let facts = crate::codebase::ts_source::facts::collect_ts_facts_with_context(
        files,
        crate::codebase::ts_source::facts::TsFactPlan {
            server_routes: true,
            ..Default::default()
        },
        &crate::codebase::ts_source::facts::TsFactContext::new(root),
    );
    facts
        .into_iter()
        .filter_map(|(path, facts)| facts.server_routes.clone().map(|routes| (path, routes)))
        .collect()
}
