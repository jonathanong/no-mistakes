use super::RULE_ID;
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_outcome::{CheckFailure, CheckInputFailure};
use crate::codebase::workflow_topology::model::{DiagnosticCode, WorkflowTopology, WorkflowTopologyDiagnostic};
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

pub(super) fn unlocated_merge_findings(topology: &WorkflowTopology) -> Vec<RuleFinding> {
    topology.diagnostics.iter()
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::UnlocatedYamlMerge)
        .map(|diagnostic| RuleFinding {
            rule: RULE_ID.to_string(),
            file: diagnostic.workflow_path.clone(),
            line: 1,
            message: format!("Cannot verify workflow topology: {}. Write the merged keys explicitly.", diagnostic.message),
            import: None,
            target: Some("unanalyzable".to_string()),
            causes: Vec::new(),
        })
        .collect()
}
