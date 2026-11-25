//! Diagnostic severity/code/shape, split out of [`super`] to stay under
//! the crate's per-file line limit. Re-exported by [`super`] so every
//! existing `model::DiagnosticCode`-style path elsewhere in the crate keeps
//! working unchanged.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticCode {
    MalformedWorkflow,
    MissingNeedsDependency,
    JobDependencyCycle,
    DuplicateStepId,
    UnknownStepReference,
    NonPriorStepReference,
    DuplicateWorkflowName,
    MissingLocalWorkflow,
    NonCallableWorkflow,
    WorkflowCallCycle,
    MissingWorkflowRunSource,
    AmbiguousWorkflowRunSource,
    WorkflowRunCycle,
    WorkflowRunChainLimit,
    UnknownWorkflowFilter,
    InvalidWorkflowFilter,
    MissingWorkflowCallInput,
    UnknownWorkflowCallInput,
    WorkflowCallInputTypeMismatch,
    MissingWorkflowCallSecret,
    UnknownWorkflowCallSecret,
    UnknownWorkflowCallOutput,
    MissingArtifactProducer,
    AmbiguousArtifactProducer,
    ArtifactResolutionLimit,
}

impl DiagnosticCode {
    /// The kebab-case wire value (matches the `#[serde(rename_all =
    /// "kebab-case")]` serialization) — used for sort-key computation
    /// without round-tripping through JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MalformedWorkflow => "malformed-workflow",
            Self::MissingNeedsDependency => "missing-needs-dependency",
            Self::JobDependencyCycle => "job-dependency-cycle",
            Self::DuplicateStepId => "duplicate-step-id",
            Self::UnknownStepReference => "unknown-step-reference",
            Self::NonPriorStepReference => "non-prior-step-reference",
            Self::DuplicateWorkflowName => "duplicate-workflow-name",
            Self::MissingLocalWorkflow => "missing-local-workflow",
            Self::NonCallableWorkflow => "non-callable-workflow",
            Self::WorkflowCallCycle => "workflow-call-cycle",
            Self::MissingWorkflowRunSource => "missing-workflow-run-source",
            Self::AmbiguousWorkflowRunSource => "ambiguous-workflow-run-source",
            Self::WorkflowRunCycle => "workflow-run-cycle",
            Self::WorkflowRunChainLimit => "workflow-run-chain-limit",
            Self::UnknownWorkflowFilter => "unknown-workflow-filter",
            Self::InvalidWorkflowFilter => "invalid-workflow-filter",
            Self::MissingWorkflowCallInput => "missing-workflow-call-input",
            Self::UnknownWorkflowCallInput => "unknown-workflow-call-input",
            Self::WorkflowCallInputTypeMismatch => "workflow-call-input-type-mismatch",
            Self::MissingWorkflowCallSecret => "missing-workflow-call-secret",
            Self::UnknownWorkflowCallSecret => "unknown-workflow-call-secret",
            Self::UnknownWorkflowCallOutput => "unknown-workflow-call-output",
            Self::MissingArtifactProducer => "missing-artifact-producer",
            Self::AmbiguousArtifactProducer => "ambiguous-artifact-producer",
            Self::ArtifactResolutionLimit => "artifact-resolution-limit",
        }
    }
}

/// Field order (severity, code, message, workflowPath, jobId?, stepIndex?,
/// callJobId?, calleeWorkflowPath?) matches every diagnostic construction
/// site in the TS engine, which happens to also match its type declaration.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTopologyDiagnostic {
    pub severity: Severity,
    pub code: DiagnosticCode,
    pub message: String,
    pub workflow_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callee_workflow_path: Option<String>,
}

impl WorkflowTopologyDiagnostic {
    /// A minimal error diagnostic; chain the `with_*` builders below for
    /// the optional fields a specific diagnostic code needs.
    pub fn new(
        code: DiagnosticCode,
        message: impl Into<String>,
        workflow_path: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Error,
            code,
            message: message.into(),
            workflow_path: workflow_path.into(),
            job_id: None,
            step_index: None,
            call_job_id: None,
            callee_workflow_path: None,
        }
    }

    pub fn with_job(mut self, job_id: impl Into<String>) -> Self {
        self.job_id = Some(job_id.into());
        self
    }

    pub fn with_step(mut self, step_index: u32) -> Self {
        self.step_index = Some(step_index);
        self
    }

    pub fn with_call_job(mut self, call_job_id: impl Into<String>) -> Self {
        self.call_job_id = Some(call_job_id.into());
        self
    }

    pub fn with_callee(mut self, callee_workflow_path: impl Into<String>) -> Self {
        self.callee_workflow_path = Some(callee_workflow_path.into());
        self
    }
}
