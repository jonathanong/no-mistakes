use super::*;

pub(super) fn collect(
    request: &GraphRuleRequest<'_>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    findings: &mut Vec<RuleFinding>,
    advisories: &mut Vec<RuleFinding>,
) -> Result<()> {
    let config = request.config;
    if !rule_enabled(config, queue_rate_limit_error_delay::RULE_ID) {
        return Ok(());
    }
    let consumer = queue_rate_limit_error_delay::RULE_ID;
    if request.dependency_graph.is_none()
        && !session.is_none_or(|session| session.check_consumer_ready(consumer))
    {
        return Ok(());
    }
    let result = (|| -> Result<()> {
        let diagnostics =
            crate::perf_trace::rule("analysis.rules.queue-rate-limit-error-delay", || {
                queue_rate_limit_error_delay::check_with_catalog(
                    request.root,
                    config,
                    request.dependency_graph,
                    request.shared,
                    no_single_enqueue_in_loop::module_catalog(
                        request.tsconfig_catalog,
                        request.sources,
                        session,
                    ),
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
    Ok(())
}
