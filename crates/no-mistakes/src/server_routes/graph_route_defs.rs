/// Project graph route definitions are projected from the invocation's
/// prepared server facts. They intentionally never fall back to raw parsing.
pub(crate) fn route_defs_from_prepared_facts_with_catalog(
    root: &Path,
    tsconfig: &TsConfig,
    tsconfig_catalog: Option<&TsConfigCatalog>,
    prepared: impl IntoIterator<Item = (PathBuf, FileFacts)>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Vec<(PathBuf, String)> {
    let root = root.canonicalize().unwrap_or(root.to_path_buf());
    let facts = prepared.into_iter().collect::<HashMap<_, _>>();
    let visible = facts.keys().cloned().collect::<HashSet<_>>();
    if let Some(catalog) = tsconfig_catalog {
        let resolver = ScopedImportResolver::new_in_session(catalog, &visible, session);
        return route_defs_from_facts_with_resolver(&root, &facts, &resolver);
    }
    let resolver = ImportResolver::new_in_session(tsconfig, Some(&visible), session);
    route_defs_from_facts_with_resolver(&root, &facts, &resolver)
}

fn route_defs_from_facts_with_resolver(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &dyn ImportResolution,
) -> Vec<(PathBuf, String)> {
    let mut route_defs = build_report_with_resolver(root, facts, None, resolver)
        .routes
        .into_iter()
        .map(|route| (root.join(route.file), route.route))
        .collect::<Vec<_>>();
    route_defs.sort();
    route_defs.dedup();
    route_defs
}
