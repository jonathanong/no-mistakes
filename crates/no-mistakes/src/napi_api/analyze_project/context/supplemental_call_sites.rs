fn supplemental_call_site_plan(
    check: Option<&SharedCheckContext>,
    primary_files: &[PathBuf],
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> ScopeFactPlan {
    let files = check
        .map(|check| check.supplemental_call_site_files().to_vec())
        .unwrap_or_default()
        .into_iter()
        // Playwright report inputs are already primary facts. Do not collect a
        // second variant when a configured call source is one of those inputs.
        .filter(|path| !primary_files.contains(path))
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
