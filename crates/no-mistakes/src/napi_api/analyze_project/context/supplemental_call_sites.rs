fn supplemental_call_site_plan(
    check: Option<&SharedCheckContext>,
    primary_files: &[PathBuf],
    graph_files: &[PathBuf],
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> ScopeFactPlan {
    let files = check
        .map(|check| check.supplemental_call_site_files().to_vec())
        .unwrap_or_default()
        .into_iter()
        // Playwright report inputs are already primary facts. Graph-only
        // inputs likewise already have richer import facts. Do not collect a
        // sparse call-site-only variant for either kind of source.
        .filter(|path| !primary_files.contains(path) && !graph_files.contains(path))
        .collect();
    ScopeFactPlan {
        files,
        graph_files: Vec::new(),
        plan: crate::codebase::check_facts::CheckFactPlan {
            graph: crate::codebase::ts_source::facts::TsFactPlan {
                call_sites: true,
                ..Default::default()
            },
            ..Default::default()
        },
        playwright: None,
        sources,
    }
}
