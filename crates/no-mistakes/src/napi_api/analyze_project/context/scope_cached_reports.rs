impl PreparedScope {
    fn queue_report(&mut self, report_type: &str, options: &ProjectOptions) -> Result<Value> {
        let key = serde_json::to_string(&options.filters)?;
        if !self.queue_reports.contains_key(&key) {
            let report = crate::queue::analyze_project_with_prepared_facts(
                self.traversal.root(),
                self.traversal.tsconfig(),
                &options.filters,
                &self.facts,
            )?;
            self.queue_reports.insert(key.clone(), report);
        }
        render_queue_report(
            report_type,
            options,
            self.queue_reports
                .get(&key)
                .expect("queue report is cached"),
        )
    }

    fn server_report(&mut self, report_type: &str, options: &ProjectOptions) -> Result<Value> {
        let prepared = self
            .server
            .as_ref()
            .context("server analysis was not prepared")?;
        let filters = server_filters(report_type, options);
        let key = serde_json::to_string(&filters)?;
        if !self.server_reports.contains_key(&key) {
            self.server_reports.insert(
                key.clone(),
                crate::server_routes::analyze_project_with_prepared(prepared, &filters)?,
            );
        }
        render_server_report(
            report_type,
            options,
            prepared,
            self.server_reports
                .get(&key)
                .expect("server report is cached"),
            &filters,
        )
    }

    fn react_report(&mut self, report_type: &str, options: &ProjectOptions) -> Result<Value> {
        if report_type == "reactUsages" {
            let target = options
                .target
                .as_deref()
                .context("target is required for react usages")?;
            let include = crate::react_traits::UsagesInclude::parse(options.include.as_deref())?;
            return Ok(serde_json::to_value(
                crate::react_traits::pipeline::usages::run_usages_with_loaded_config_and_facts(
                    self.traversal.root(),
                    self.traversal.config(),
                    target,
                    &options.targets,
                    &include,
                    &self.facts,
                )?,
            )?);
        }
        let key = serde_json::to_string(&options.targets)?;
        if !self.react_analyses.contains_key(&key) {
            self.react_analyses.insert(
                key.clone(),
                crate::react_traits::pipeline::run_with_facts::run_analyze_with_loaded_config_and_facts(
                    self.traversal.root(),
                    self.traversal.config(),
                    &options.targets,
                    &self.facts,
                )?,
            );
        }
        if report_type == "reactAnalyze" {
            return Ok(serde_json::to_value(
                self.react_analyses
                    .get(&key)
                    .expect("React analysis is cached"),
            )?);
        }
        let prepared = crate::react_traits::prepare_check_from_loaded_config(
            self.traversal.config(),
            options.assert_no_fetch,
        );
        Ok(serde_json::to_value(
            crate::react_traits::run_check_with_prepared_facts(
                self.traversal.root(),
                &options.targets,
                &self.facts,
                &prepared,
            )?,
        )?)
    }
}
