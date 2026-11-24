use super::CompletedDomainChecks;
use std::path::Path;

pub(super) struct CollectedWarnings {
    pub(super) warnings: Vec<String>,
    pub(super) analysis_warning_count: usize,
}

pub(super) fn collect(
    root: &Path,
    facts: &no_mistakes::codebase::check_facts::CheckFactMap,
    config: &no_mistakes::config::v2::NoMistakesConfig,
    react_warning: Option<String>,
    completed: &mut CompletedDomainChecks,
) -> CollectedWarnings {
    let mut warnings: Vec<String> = [
        react_warning,
        completed.react.warning.take(),
        completed.queues.warning.take(),
        completed.rules.warning.take(),
        completed.integration.warning.take(),
        completed.codebase.warning.take(),
        completed.filesystem_rules.warning.take(),
    ]
    .into_iter()
    .flatten()
    .collect();
    let executor_warnings = no_mistakes::codebase::postgres::executor_config_warnings(root, facts);
    let task_warnings: Vec<String> = [
        std::mem::take(&mut completed.react.analysis_warnings),
        std::mem::take(&mut completed.queues.analysis_warnings),
        std::mem::take(&mut completed.rules.analysis_warnings),
        std::mem::take(&mut completed.integration.analysis_warnings),
        std::mem::take(&mut completed.codebase.analysis_warnings),
        std::mem::take(&mut completed.filesystem_rules.analysis_warnings),
    ]
    .into_iter()
    .flatten()
    .collect();
    let analysis_warning_count =
        config.analysis_warnings.len() + executor_warnings.len() + task_warnings.len();
    warnings.extend(executor_warnings);
    warnings.extend(task_warnings);
    warnings.extend(config.analysis_warnings.iter().cloned());
    CollectedWarnings {
        warnings,
        analysis_warning_count,
    }
}
