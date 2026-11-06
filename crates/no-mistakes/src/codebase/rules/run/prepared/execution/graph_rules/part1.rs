use super::*;

pub(super) fn collect(
    request: &GraphRuleRequest<'_>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    findings: &mut Vec<RuleFinding>,
    advisories: &mut Vec<RuleFinding>,
) -> Result<()> {
    let root = request.root;
    let config = request.config;
    let config_path = request.config_path;
    let shared = request.shared;
    let prepared_graph = request.prepared_graph;
    let dependency_graph = request.dependency_graph;
    let inferred_roots = request.inferred_roots;
    let sources = request.sources;
    if rule_enabled(config, FORBIDDEN_DEPENDENCIES) {
        let consumer = "forbidden-dependencies";
        if dependency_graph.is_some()
            || session.is_none_or(|session| session.check_consumer_ready(consumer))
        {
            let result = (|| -> Result<()> {
                findings.extend(crate::perf_trace::rule(
                    "analysis.rules.forbidden-dependencies",
                    || {
                        forbidden_dependencies::check_with_prepared_facts_and_graph(
                            root,
                            config,
                            forbidden_dependencies::PreparedConfig {
                                path: config_path,
                                sources,
                            },
                            shared,
                            prepared_graph,
                            inferred_roots,
                            dependency_graph
                                .expect("forbidden-dependencies requires canonical graph"),
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
    if rule_enabled(config, REQUIRED_ENTRYPOINT_REACHABILITY) {
        let consumer = "required-entrypoint-reachability";
        if dependency_graph.is_some()
            || session.is_none_or(|session| session.check_consumer_ready(consumer))
        {
            let result = (|| -> Result<()> {
                findings.extend(crate::perf_trace::rule(
                    "analysis.rules.required-entrypoint-reachability",
                    || {
                        required_entrypoint_reachability::check_with_graph_and_inferred(
                            root,
                            config,
                            shared.graph_file_universe(),
                            dependency_graph.expect(
                                "required-entrypoint-reachability requires canonical graph",
                            ),
                            inferred_roots,
                            sources,
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
    if rule_enabled(config, UNCONSTRUCTED_ERROR_CLASS) {
        let consumer = "unconstructed-error-class";
        if dependency_graph.is_some()
            || session.is_none_or(|session| session.check_consumer_ready(consumer))
        {
            let result = (|| -> Result<()> {
                let diagnostics =
                    crate::perf_trace::rule("analysis.rules.unconstructed-error-class", || {
                        unconstructed_error_class::check_with_graph(
                            root,
                            config,
                            dependency_graph
                                .expect("unconstructed-error-class requires canonical graph"),
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
