fn prepare_playwright_views(
    options: &AnalyzeProjectOptions,
    traversal: &SharedTraversalContext,
    check: Option<&SharedCheckContext>,
) -> Result<HashMap<String, PreparedPlaywrightView>> {
    let mut views = HashMap::new();
    for request in options.reports.iter().filter(|request| {
        matches!(
            request.report_type.as_str(),
            "playwrightCheck" | "playwrightEdges" | "playwrightRelated" | "playwrightTests"
        )
    }) {
        let raw = playwright_options(request, options)?;
        let parsed: PlaywrightOptions = serde_json::from_str(&raw)?;
        let key = playwright_analysis_key(&parsed)?;
        if views.contains_key(&key) {
            continue;
        }
        let prepared = check.and_then(|check| check.playwright_report_view(&parsed));
        let view = match prepared {
            Some(view) => view,
            None => {
                let root = traversal.root();
                let playwright_configs = parsed
                    .playwright_config
                    .iter()
                    .map(PathBuf::from)
                    .collect::<Vec<_>>();
                let settings = crate::playwright::config::settings_from_loaded_v2(
                    root,
                    traversal.config(),
                    &playwright_configs,
                    parsed.project.clone(),
                    traversal.visible_paths(),
                )?;
                let mut fact_plan = crate::playwright::analysis::pipeline::standalone_fact_plan(
                    root,
                    &settings,
                    playwright_unique_policy(&parsed),
                    traversal.visible_paths(),
                )?;
                fact_plan.configure_module_resolution_with_catalog(
                    traversal.tsconfig_catalog_arc(),
                    std::sync::Arc::new(traversal.workspace().clone()),
                    traversal.visible_paths(),
                    root,
                );
                PreparedPlaywrightView {
                    settings,
                    fact_plan,
                }
            }
        };
        views.insert(key, view);
    }
    Ok(views)
}

fn same_config_path(root: &Path, left: Option<&Path>, right: Option<&Path>) -> bool {
    let normalize = |path: &Path| {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        crate::codebase::ts_resolver::normalize_path(&path)
    };
    left.map(normalize) == right.map(normalize)
}

fn playwright_analysis_key(options: &PlaywrightOptions) -> Result<String> {
    let mut options = options.clone();
    options.files.clear();
    Ok(serde_json::to_string(&options)?)
}

fn playwright_unique_policy(
    options: &PlaywrightOptions,
) -> crate::playwright::analysis::types::UniqueSelectorPolicy {
    crate::playwright::analysis::types::UniqueSelectorPolicy {
        test_ids: options.assert_unique_test_ids,
        html_ids: options.assert_unique_html_ids,
        aggregate: false,
        configured_html_id_selector: false,
    }
}
