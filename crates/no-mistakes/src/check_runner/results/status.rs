use crate::check_runner::CheckResults;

pub(crate) fn has_failures(results: &CheckResults) -> bool {
    !results.incomplete.is_empty()
        || !results.react.is_empty()
        || !results.queues.is_empty()
        || !results.rules.is_empty()
        || !results.integration.is_empty()
        || !results.codebase.is_empty()
        || results.warnings.len() > results.analysis_warning_count
}
