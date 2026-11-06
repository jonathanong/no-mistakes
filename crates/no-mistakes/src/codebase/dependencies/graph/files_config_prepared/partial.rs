impl PreparedGraphConfig {
    #[doc(hidden)]
    pub fn playwright_fact_plan_partial(
        &mut self,
        root: &Path,
        tsconfig: &crate::codebase::ts_resolver::TsConfig,
        visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    ) -> (
        Option<crate::codebase::check_facts::PlaywrightFactPlan>,
        Vec<PlaywrightPreparationFailure>,
    ) {
        for settings in &mut self.playwright_settings {
            for path in &mut settings.playwright_configs {
                *path = crate::codebase::ts_resolver::normalize_path(&root.join(&*path));
            }
        }
        let sources = visible_paths.source_store_for(root);
        let mut failures = std::mem::take(&mut self.playwright_preparation_failures);
        let mut paths = self
            .playwright_settings
            .iter()
            .flat_map(|settings| settings.playwright_configs.iter().cloned())
            .collect::<Vec<_>>();
        paths.sort();
        paths.dedup();
        let mut failed_paths = crate::fx::FxHashSet::default();
        let mut loaded = Vec::new();
        for path in paths {
            match crate::playwright::playwright_config::load_partial_with_sources(
                root, &path, &sources,
            ) {
                Ok((config, errors)) => {
                    for (project, error) in errors {
                        for settings in self
                            .playwright_settings
                            .iter()
                            .filter(|settings| settings.playwright_configs.contains(&path))
                        {
                            failures.push(PlaywrightPreparationFailure::new(
                                root,
                                settings,
                                Some(path.clone()),
                                project.clone(),
                                config.name.clone(),
                                anyhow::anyhow!("{error:#}"),
                            ));
                        }
                    }
                    loaded.push((path.clone(), config));
                }
                Err(error) => {
                    failed_paths.insert(path.clone());
                    let config_name =
                        crate::playwright::playwright_config::config_name_with_sources(
                            &path, &sources,
                        );
                    for settings in self
                        .playwright_settings
                        .iter()
                        .filter(|settings| settings.playwright_configs.contains(&path))
                    {
                        failures.push(PlaywrightPreparationFailure::new(
                            root,
                            settings,
                            Some(path.clone()),
                            None,
                            config_name.clone(),
                            anyhow::anyhow!("{error:#}"),
                        ));
                    }
                }
            }
        }
        let mut merged = None;
        self.playwright_settings.retain_mut(|settings| {
            let had_configs = !settings.playwright_configs.is_empty();
            settings
                .playwright_configs
                .retain(|path| !failed_paths.contains(path));
            if had_configs && settings.playwright_configs.is_empty() {
                return false;
            }
            let mut plan = crate::codebase::check_facts::PlaywrightFactPlan::default();
            let outcome = crate::playwright::playwright_config::select_loaded(
                root,
                &settings.playwright_configs,
                settings.project.as_deref(),
                &loaded,
            )
            .and_then(|playwright| {
                crate::playwright::analysis::pipeline::extend_fact_plan_with_config(
                    &mut plan,
                    root,
                    settings,
                    crate::playwright::analysis::types::UniqueSelectorPolicy::default(),
                    visible_paths,
                    &playwright,
                )
            });
            match outcome {
                Ok(()) => {
                    merged
                        .get_or_insert_with(
                            crate::codebase::check_facts::PlaywrightFactPlan::default,
                        )
                        .include(plan);
                    true
                }
                Err(error) => {
                    failures.push(PlaywrightPreparationFailure::new(
                        root,
                        settings,
                        None,
                        settings.project.clone(),
                        None,
                        error,
                    ));
                    false
                }
            }
        });
        if self.playwright_settings.is_empty() && !failures.is_empty() {
            self.disable_playwright();
        }
        if let Some(plan) = merged.as_mut() {
            plan.configure_module_resolution(
                std::sync::Arc::new(tsconfig.clone()),
                std::sync::Arc::clone(&self.workspace),
                visible_paths,
                root,
            );
        }
        (merged, failures)
    }
}
