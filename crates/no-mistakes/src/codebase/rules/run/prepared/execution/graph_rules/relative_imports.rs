use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::rules::no_cross_package_relative_imports as relative_imports;

pub(super) fn collect(
    request: &GraphRuleRequest<'_>,
    session: Option<&AnalysisSession>,
    findings: &mut Vec<RuleFinding>,
    advisories: &mut Vec<RuleFinding>,
) -> Result<()> {
    let Some(graph) = request
        .dependency_graph
        .filter(|_| request.config.rule_configured(relative_imports::RULE_ID))
    else {
        return Ok(());
    };
    let session = session.expect("relative import analysis requires an analysis session");
    let owned_workspace;
    let workspace = if let Some(prepared) = request.prepared_graph {
        prepared.workspace()
    } else {
        owned_workspace = session.workspace(request.root);
        &owned_workspace
    };
    let result =
        crate::perf_trace::rule("analysis.rules.no-cross-package-relative-imports", || {
            relative_imports::check(
                request.root,
                request.config,
                request.shared,
                graph,
                workspace,
                session,
            )
        });
    if let Some(output) = session.capture_check_result(relative_imports::RULE_ID, result)? {
        findings.extend(output.findings);
        advisories.extend(output.advisories);
    }
    Ok(())
}
