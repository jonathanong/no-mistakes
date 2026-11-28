use super::*;
use crate::codebase::rules::coverage_producer_assertion_owner::{check, RULE_ID};

pub(super) fn collect(
    request: &GraphRuleRequest<'_>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    findings: &mut Vec<RuleFinding>,
    advisories: &mut Vec<RuleFinding>,
) -> Result<()> {
    super::collect_diagnostics_rule(
        request,
        session,
        RULE_ID,
        "analysis.rules.coverage-producer-assertion-owner",
        |graph| {
            check(
                request.root,
                request.config,
                graph,
                request.shared,
                request
                    .sources
                    .expect("coverage ownership requires prepared sources"),
            )
        },
        findings,
        advisories,
    )
}
