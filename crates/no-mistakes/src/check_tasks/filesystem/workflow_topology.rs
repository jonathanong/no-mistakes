use anyhow::Result;
use no_mistakes::codebase::rules::{self, analysis_policy::AnalysisDiagnostics};
use no_mistakes::config::v2::NoMistakesConfig;
use std::borrow::Cow;
use std::path::Path;

pub(super) fn run<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    prepared: &rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    defer_suppression: bool,
) -> Result<(Cow<'a, NoMistakesConfig>, AnalysisDiagnostics)> {
    let mut remaining = Cow::Borrowed(config);
    let mut output = AnalysisDiagnostics::default();
    if !config.rule_configured(rules::WORKFLOW_TOPOLOGY_POLICY) {
        return Ok((remaining, output));
    }
    let Some(inputs) = prepared.workflow_documents else {
        return Ok((remaining, output));
    };
    let result = no_mistakes::perf_trace::filesystem_rule(rules::WORKFLOW_TOPOLOGY_POLICY, || {
        rules::workflow_topology_policy::diagnostics_prepared(
            root,
            config,
            prepared.session,
            &prepared.sources,
            inputs,
            prepared.config_path,
            defer_suppression,
        )
    });
    if let Some(session) = prepared.session {
        if let Some(chunk) = session.capture_check_result(rules::WORKFLOW_TOPOLOGY_POLICY, result)?
        {
            output.append(chunk);
        }
    } else {
        output.append(result?);
    }
    super::handled::mark(remaining.to_mut(), |rule| {
        rule == rules::WORKFLOW_TOPOLOGY_POLICY
    });
    Ok((remaining, output))
}
