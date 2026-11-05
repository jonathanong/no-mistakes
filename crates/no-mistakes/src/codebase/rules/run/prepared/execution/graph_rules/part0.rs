use super::*;

pub(super) fn collect(
    request: &GraphRuleRequest<'_>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    findings: &mut Vec<RuleFinding>,
    advisories: &mut Vec<RuleFinding>,
) -> Result<()> {
    let root = request.root;
    let config = request.config;
    let shared = request.shared;
    let prepared_graph = request.prepared_graph;
    let dependency_graph = request.dependency_graph;
    let tsconfig_catalog = request.tsconfig_catalog;
    let prepared_vitest_projects = request.prepared_vitest_projects;
    let prepared_playwright_projects = request.prepared_playwright_projects;
    if config.rule_configured(crate::codebase::rules::PRODUCTION_EXPORTS_ONLY_USED_BY_TESTS) {
        let consumer = "production-exports-only-used-by-tests";
        if dependency_graph.is_some()
            || session.is_none_or(|session| session.check_consumer_ready(consumer))
        {
            let result = (|| -> Result<()> {
                let diagnostics = crate::perf_trace::rule(
                    "analysis.rules.production-exports-only-used-by-tests",
                    || {
                        crate::codebase::rules::production_exports_test_only::check_with_graph_and_session(
                            root,
                            config,
                            dependency_graph.expect(
                                "production-exports-only-used-by-tests requires canonical graph",
                            ),
                            shared,
                            session,
                        )
                    },
                )?;
                findings.extend(diagnostics.findings);
                advisories.extend(diagnostics.advisories);

                Ok(())
            })();
            if let Some(session) = session {
                session.capture_check_result(consumer, result)?;
            } else {
                result?;
            }
        }
    }
    if rule_enabled(config, FORBIDDEN_CALLS) {
        let consumer = "forbidden-calls";
        if dependency_graph.is_some()
            || session.is_none_or(|session| session.check_consumer_ready(consumer))
        {
            let result = (|| -> Result<()> {
                findings.extend(crate::perf_trace::rule(
                    "analysis.rules.forbidden-calls",
                    || {
                        forbidden_calls::check_with_graph(
                            root,
                            config,
                            dependency_graph.expect("forbidden-calls requires canonical graph"),
                            prepared_vitest_projects,
                            prepared_playwright_projects,
                            shared.graph_file_universe(),
                            Some(shared),
                        )
                    },
                )?);

                Ok(())
            })();
            if let Some(session) = session {
                session.capture_check_result(consumer, result)?;
            } else {
                result?;
            }
        }
    }
    if rule_enabled(
        config,
        crate::codebase::rules::test_no_live_repository_scan::RULE_ID,
    ) {
        let consumer = "test-no-live-repository-scan";
        if dependency_graph.is_some()
            || session.is_none_or(|session| session.check_consumer_ready(consumer))
        {
            let result = (|| -> Result<()> {
                let diagnostics =
                    crate::perf_trace::rule("analysis.rules.test-no-live-repository-scan", || {
                        crate::codebase::rules::test_no_live_repository_scan::check_with_graph(
                            root,
                            config,
                            dependency_graph
                                .expect("test-no-live-repository-scan requires canonical graph"),
                            prepared_vitest_projects,
                            shared.graph_file_universe(),
                            shared,
                            &Resolution::new(tsconfig_catalog, prepared_graph),
                        )
                    })?;
                findings.extend(diagnostics.findings);
                advisories.extend(diagnostics.advisories);

                Ok(())
            })();
            if let Some(session) = session {
                session.capture_check_result(consumer, result)?;
            } else {
                result?;
            }
        }
    }
    Ok(())
}
