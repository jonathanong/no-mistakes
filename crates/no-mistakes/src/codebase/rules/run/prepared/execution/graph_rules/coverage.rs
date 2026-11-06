use super::*;
use crate::codebase::rules::coverage_producer_assertion_owner::{check, RULE_ID};

pub(super) fn collect(
    request: &GraphRuleRequest<'_>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    findings: &mut Vec<RuleFinding>,
    advisories: &mut Vec<RuleFinding>,
) -> Result<()> {
    if !request.config.rule_configured(RULE_ID)
        || (request.dependency_graph.is_none()
            && session.is_some_and(|session| !session.check_consumer_ready(RULE_ID)))
    {
        return Ok(());
    }
    let result =
        crate::perf_trace::rule("analysis.rules.coverage-producer-assertion-owner", || {
            check(
                request.root,
                request.config,
                request
                    .dependency_graph
                    .expect("coverage ownership requires canonical graph"),
                request.shared,
                request
                    .sources
                    .expect("coverage ownership requires prepared sources"),
            )
        });
    let output = match session {
        Some(session) => session.capture_check_result(RULE_ID, result)?,
        None => Some(result?),
    };
    if let Some(output) = output {
        findings.extend(output.findings);
        advisories.extend(output.advisories);
    }
    Ok(())
}
