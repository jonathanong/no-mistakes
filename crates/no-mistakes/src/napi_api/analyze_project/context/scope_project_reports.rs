impl PreparedScope {
    pub(super) fn project_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let raw = project_options(request, options)?;
        let parsed: ProjectOptions = serde_json::from_str(&raw)?;
        match request.report_type.as_str() {
            "queues" | "queueEdges" | "queueRelated" | "queueCheck" => {
                self.queue_report(&request.report_type, &parsed)
            }
            value if super::is_server_report(value) => self.server_report(value, &parsed),
            "reactAnalyze" | "reactCheck" | "reactUsages" => {
                self.react_report(&request.report_type, &parsed)
            }
            "check" => {
                let dependency_graph = if self
                    .check
                    .as_ref()
                    .and_then(SharedCheckContext::graph_plan)
                    .is_some()
                {
                    Some(self.traversal.canonical_graph()?)
                } else {
                    None
                };
                let check = self
                    .check
                    .as_ref()
                    .context("check analysis was not prepared")?;
                Ok(crate::check_runner::json_value(
                    &check.run(&self.facts, dependency_graph.as_ref())?,
                ))
            }
            _ => unreachable!("project report types are checked before dispatch"),
        }
    }

    pub(super) fn playwright_report(
        &mut self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Value> {
        let raw = playwright_options(request, options)?;
        let parsed: PlaywrightOptions = serde_json::from_str(&raw)?;
        let key = playwright_analysis_key(&parsed)?;
        let Some(prepared) = self.playwright.get(&key) else {
            bail!(
                "distinct Playwright settings require a separate prepared analyzeProject context"
            );
        };
        if !self.playwright_analyses.contains_key(&key) {
            let analysis =
                crate::playwright::analysis::pipeline::analyze_with_policy_and_facts_from_snapshot(
                    self.traversal.root(),
                    &prepared.settings,
                    crate::playwright::playwright_tests::TestPolicy {
                        assert_conditional_tests: parsed.assert_conditional_tests,
                        allow_skipped_tests: parsed.allow_skipped_tests,
                    },
                    playwright_unique_policy(&parsed),
                    &self.facts,
                    self.traversal.visible_paths(),
                )?;
            self.playwright_analyses.insert(key.clone(), analysis);
        }
        let analysis = self
            .playwright_analyses
            .get(&key)
            .expect("Playwright analysis is cached");
        render_playwright_report(
            &request.report_type,
            &parsed,
            self.traversal.root(),
            analysis,
        )
    }
}
