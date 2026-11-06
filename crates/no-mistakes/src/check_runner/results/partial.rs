use super::{CheckTask, CompletedDomainChecks, DomainResults};
use no_mistakes::codebase::analysis_session::AnalysisSession;
use no_mistakes::codebase::check_outcome::CheckFailure;
use std::time::Duration;

pub(crate) fn retain_domain_checks(
    session: &AnalysisSession,
    results: DomainResults,
) -> CompletedDomainChecks {
    let (react, queues, rules, integration, codebase, filesystem_rules) = results;
    CompletedDomainChecks {
        react: retain(session, "react", react),
        queues: retain(session, "queues", queues),
        rules: retain(session, "rules", rules),
        integration: retain(session, "integration", integration),
        codebase: retain(session, "codebase", codebase),
        filesystem_rules: retain(session, "filesystem", filesystem_rules),
    }
}

fn retain<T: Default>(
    session: &AnalysisSession,
    consumer: &str,
    result: anyhow::Result<CheckTask<T>>,
) -> CheckTask<T> {
    match result {
        Ok(task) => {
            if let Some(warning) = &task.warning {
                session.record_check_failure(CheckFailure::new(consumer, warning.clone()));
            }
            task
        }
        Err(error) => {
            session.record_check_failure(CheckFailure::new(consumer, format!("{error:#}")));
            CheckTask {
                findings: T::default(),
                react_suppression_targets: Vec::new(),
                pre_suppressed: Vec::new(),
                suppression_sources: Vec::new(),
                advisories: Vec::new(),
                warning: None,
                duration: Duration::ZERO,
            }
        }
    }
}
