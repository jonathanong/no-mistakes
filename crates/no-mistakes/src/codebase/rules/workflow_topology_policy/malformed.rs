use super::RULE_ID;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_outcome::{CheckFailure, CheckInputFailure};
use crate::codebase::workflow_topology::model::WorkflowTopologyDiagnostic;
use std::path::PathBuf;

pub(super) fn report_malformed(
    diagnostic: &WorkflowTopologyDiagnostic,
    session: Option<&AnalysisSession>,
) -> anyhow::Result<()> {
    let file = PathBuf::from(&diagnostic.workflow_path);
    let message = format!(
        "Cannot verify reusable workflow concurrency: {}. Repair the workflow YAML and run the check again.",
        diagnostic.message
    );
    let Some(session) = session else {
        return Err(CheckInputFailure::source(file, message).into());
    };
    session.record_check_failure(CheckFailure {
        consumer: RULE_ID.to_owned(),
        file: Some(file),
        property: None,
        message,
    });
    Ok(())
}
