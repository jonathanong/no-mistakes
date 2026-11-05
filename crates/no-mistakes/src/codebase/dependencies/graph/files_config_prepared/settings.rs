fn prepare_playwright_settings(
    root: &Path,
    plan: GraphBuildPlan,
    config: &crate::config::v2::NoMistakesConfig,
    visible: &crate::codebase::ts_source::VisiblePathSnapshot,
    partial: bool,
) -> anyhow::Result<(
    Vec<crate::playwright::config::Settings>,
    Vec<PlaywrightPreparationFailure>,
)> {
    if !(plan.playwright_routes || plan.playwright_selectors) {
        return Ok((Vec::new(), Vec::new()));
    }
    if !crate::playwright::config::has_v2_playwright_settings(config) {
        return Ok((
            vec![crate::playwright::config::settings_from_loaded_v2(
                root,
                config,
                &[],
                None,
                None,
                visible,
            )?],
            Vec::new(),
        ));
    }
    let apps = crate::config::v2::frontend_apps_or_default(root, config, &visible.paths_for(root))?;
    let mut settings = Vec::new();
    let mut failures = Vec::new();
    for app in apps {
        match crate::playwright::config::settings_from_loaded_v2(
            root,
            config,
            &[],
            None,
            app.project.clone(),
            visible,
        ) {
            Ok(value) => settings.push(value),
            Err(error) if partial => failures.push(PlaywrightPreparationFailure {
                file: None,
                project: None,
                config_name: None,
                message: format!("{error:#}"),
                roots: std::iter::once(&app.route_root)
                    .chain(&app.selector_roots)
                    .map(|path| crate::codebase::ts_resolver::normalize_path(&root.join(path)))
                    .collect(),
            }),
            Err(error) => return Err(error),
        }
    }
    Ok((settings, failures))
}
