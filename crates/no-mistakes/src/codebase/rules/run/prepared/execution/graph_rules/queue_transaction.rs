use super::*;
use crate::codebase::rules::queue_no_enqueue_in_transaction::{check, RULE_ID};

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
        "analysis.rules.queue-no-enqueue-in-transaction",
        |graph| check(request.root, request.config, graph, request.shared),
        findings,
        advisories,
    )
}
