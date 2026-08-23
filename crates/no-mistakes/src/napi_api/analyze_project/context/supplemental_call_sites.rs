fn supplemental_call_site_plan(
    check: Option<&SharedCheckContext>,
    primary_files: &[PathBuf],
    graph_files: &[PathBuf],
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> ScopeFactPlan {
    let files = check
        .map(|check| check.supplemental_call_site_files().to_vec())
        .unwrap_or_default();
    // Playwright report inputs are already primary facts. Graph-only inputs
    // likewise already have richer import facts. Move retained paths out of
    // the temporary context projection instead of cloning them twice.
    let files = crate::codebase::check_facts::into_ordered_path_exclusion(
        files,
        primary_files,
        graph_files,
    );
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
