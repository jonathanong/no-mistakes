impl PreparedScope {
    pub(super) fn project_report(
        &self,
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
                let check = self
                    .check
                    .as_ref()
                    .context("check analysis was not prepared")?;
                let dependency_graph = if check.graph_plan().is_some()
                    && self.check_uses_traversal_graph
                {
                    Some(self.traversal.graph_shared()?)
                } else {
                    None
                };
                Ok(crate::check_runner::json_value(&check.run(
                    &self.check_facts,
                    dependency_graph.as_ref(),
                    self.traversal.session_arc(),
                    parsed.include_suppressed,
                )?))
            }
            _ => unreachable!("project report types are checked before dispatch"),
        }
    }

    pub(super) fn playwright_report(
        &self,
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
        let cached = self
            .playwright_analyses
            .lock()
            .expect("playwright analysis cache is poisoned")
            .get(&key)
            .cloned();
        let analysis = match cached {
            Some(analysis) => analysis,
            None => {
                let computed = std::sync::Arc::new(
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
                    )?,
                );
                self.playwright_analyses
                    .lock()
                    .expect("playwright analysis cache is poisoned")
                    .entry(key)
                    .or_insert_with(|| computed.clone())
                    .clone()
            }
        };
        render_playwright_report(
            &request.report_type,
            &parsed,
            self.traversal.root(),
            analysis.as_ref(),
        )
    }
}
