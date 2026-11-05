use super::*;
use no_mistakes::codebase::rules::analysis_policy::AnalysisDiagnostics;
use rayon::prelude::*;

pub(super) fn run(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    prepared: rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    facts: Option<&no_mistakes::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let session = prepared
        .session
        .expect("partial dispatch requires a session");
    let mut consumers: Vec<_> = config
        .rules
        .iter()
        .filter(|rule| rule.enabled && FILESYSTEM_RULE_IDS.contains(&rule.rule.as_str()))
        .map(|rule| rule.rule.as_str())
        .collect();
    consumers.sort_unstable();
    consumers.dedup();
    let chunks = consumers
        .par_iter()
        .map(|consumer| {
            let mut selected = config.clone();
            for rule in &mut selected.rules {
                rule.enabled &= rule.rule == *consumer;
            }
            let inputs = rules::filesystem_dispatch::PreparedFilesystemRuleInputs {
                session: Some(session),
                next_config_resolver: prepared.next_config_resolver,
                snapshot: prepared.snapshot,
                vitest_catalog: prepared.vitest_catalog,
                sources: prepared.sources.clone(),
                workflow_documents: prepared.workflow_documents,
                tsconfig_gate_project_inputs: prepared.tsconfig_gate_project_inputs,
                config_path: prepared.config_path,
            };
            session.capture_check_result(
                consumer,
                super::finite_sets::run(
                    root,
                    &selected,
                    true,
                    files,
                    inputs,
                    facts,
                    defer_suppression,
                ),
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let mut output = AnalysisDiagnostics::default();
    for chunk in chunks.into_iter().flatten() {
        output.append(chunk);
    }
    output.findings.sort();
    output.advisories.sort();
    Ok(output)
}
