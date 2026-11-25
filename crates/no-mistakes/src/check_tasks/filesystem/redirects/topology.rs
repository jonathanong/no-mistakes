use super::*;

pub(super) fn run<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    prepared: &rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    defer_suppression: bool,
    output: &mut AnalysisDiagnostics,
) -> Result<std::borrow::Cow<'a, NoMistakesConfig>> {
    if !config.rule_configured(rules::WORKFLOW_TOPOLOGY_POLICY) {
        return Ok(std::borrow::Cow::Borrowed(config));
    }
    let result = no_mistakes::perf_trace::filesystem_rule(rules::WORKFLOW_TOPOLOGY_POLICY, || {
        let inputs = prepared.workflow_documents.ok_or_else(|| {
            anyhow::anyhow!("prepared workflow documents are required for workflow-topology-policy")
        })?;
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
    let diagnostics = match prepared.session {
        Some(session) => session
            .capture_check_result(rules::WORKFLOW_TOPOLOGY_POLICY, result)?
            .unwrap_or_default(),
        None => result?,
    };
    output.append(diagnostics);
    let mut remaining = config.clone();
    super::super::handled::mark(&mut remaining, |id| id == rules::WORKFLOW_TOPOLOGY_POLICY);
    Ok(std::borrow::Cow::Owned(remaining))
}
