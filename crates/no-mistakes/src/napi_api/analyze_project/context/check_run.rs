impl SharedCheckContext {
    fn fact_plan(&self) -> crate::codebase::check_facts::CheckFactPlan {
        self.plan.clone()
    }

    fn playwright_fact_plan(&self) -> Option<crate::codebase::check_facts::PlaywrightFactPlan> {
        self.playwright_fact_plan.clone()
    }

    fn playwright_report_view(
        &self,
        options: &PlaywrightOptions,
    ) -> Option<PreparedPlaywrightView> {
        if !options.playwright_config.is_empty()
            || !same_config_path(
                &self.root,
                self.config_path.as_deref(),
                options.config.as_deref().map(Path::new),
            )
        {
            return None;
        }
        let (settings, fact_plan) = self
            .prepared
            .playwright
            .as_ref()?
            .report_view(options.project.as_deref(), options.assert_unique_html_ids)?;
        Some(PreparedPlaywrightView {
            settings,
            fact_plan,
        })
    }

    fn fact_files(&self) -> &[PathBuf] {
        &self.fact_files
    }

    fn graph_files(&self) -> &[PathBuf] {
        &self.graph_files
    }

    fn graph_plan(&self) -> Option<crate::codebase::dependencies::graph::GraphBuildPlan> {
        self.graph_plan
    }

    fn run(
        &self,
        facts: &crate::codebase::check_facts::CheckFactMap,
        dependency_graph: Option<&std::sync::Arc<crate::codebase::dependencies::graph::DepGraph>>,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    ) -> Result<crate::check_runner::CheckResults> {
        use crate::check_parallel::{run_domain_checks, DomainCheckInputs};
        use crate::codebase::rules::agents_md_max_size::advisories_with_files_and_sources;

        if self.fact_files.is_empty()
            && self.graph_files.is_empty()
            && !self.filesystem_rules_enabled
            && !self.playwright_rules_enabled
            && !self.forbidden_deps_enabled
        {
            return Ok(crate::check_runner::empty_results([None]));
        }
        let config = &self.prepared.config;
        let sources = self.prepared.visible_paths.source_store_for(&self.root);
        let (react, queues, rules, integration, codebase, filesystem_rules) =
            run_domain_checks(DomainCheckInputs {
                session,
                root: &self.root,
                config_path: &self.config_path,
                tsconfig_path: &self.tsconfig_path,
                react_enabled: self.react_enabled,
                queues_enabled: self.queues_enabled,
                unique_exports_enabled: self.unique_exports_enabled,
                filesystem_rules_enabled: self.filesystem_rules_enabled,
                discovered_files: self.fs_files.clone(),
                facts,
                prepared_playwright: self.prepared.playwright.as_ref(),
                prepared_react: &self.prepared.react,
                prepared_graph: self.prepared_graph.as_ref(),
                dependency_graph: dependency_graph.cloned(),
                prepared_tsconfig: &self.prepared.tsconfig,
                prepared_tsconfig_catalog: &self.prepared.tsconfig_catalog,
                visible_paths: self.prepared.visible_paths.as_ref(),
                sources: std::sync::Arc::clone(&sources),
                inferred_roots: &self.prepared.inferred_roots,
                config,
                codebase_config: &self.prepared.codebase_config,
                vitest_projects: self.prepared.vitest_projects.as_ref(),
            });
        let completed = crate::check_runner::complete_domain_checks((
            react,
            queues,
            rules,
            integration,
            codebase,
            filesystem_rules,
        ))?;
        let mut rules = completed.rules.findings;
        rules.extend(completed.filesystem_rules.findings);
        let warnings = [
            completed.react.warning,
            completed.queues.warning,
            completed.rules.warning,
            completed.integration.warning,
            completed.codebase.warning,
            completed.filesystem_rules.warning,
        ]
        .into_iter()
        .flatten()
        .collect();
        let advisories = if self.filesystem_rules_enabled {
            advisories_with_files_and_sources(&self.root, config, &self.fs_files, &sources)?
        } else {
            Vec::new()
        };
        Ok(crate::check_runner::CheckResults {
            timings: vec![
                ("discover", std::time::Duration::ZERO),
                ("parse_extract", std::time::Duration::ZERO),
                ("react", completed.react.duration),
                ("queues", completed.queues.duration),
                ("rules", completed.rules.duration),
                ("integration", completed.integration.duration),
                ("codebase", completed.codebase.duration),
                ("filesystem_rules", completed.filesystem_rules.duration),
            ],
            react: completed.react.findings,
            queues: completed.queues.findings,
            rules,
            integration: completed.integration.findings,
            codebase: completed.codebase.findings,
            warnings,
            advisories,
        })
    }
}
