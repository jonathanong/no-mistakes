//! `no-mistakes ci env <VAR>` — find every workflow definition and
//! `${{ env.VAR }}` reference of an environment variable.
//!
//! Definitions are read from structured `env:` blocks at the workflow, job, and
//! step scopes. References are a textual scan of every string scalar for a
//! `${{ … env.VAR … }}` expression, attributed to the nearest enclosing scope.
//! Matching is case-sensitive (Linux runner semantics) and does not resolve
//! computed expressions. Exact line numbers are intentionally omitted — use
//! `rg 'env.VAR' <file>` for those.

use super::model::CiWarning;
use crate::codebase::ci_workflows::{ParsedWorkflowSet, WorkflowDocumentErrorKind};
use crate::codebase::ts_source::VisiblePathSnapshot;
use crate::config::v2::schema::CiConfig;
use serde::Serialize;
use std::path::Path;

mod collect;

use collect::{collect_locations, reference_regex};

/// Result of an `env` query across the workflow set.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CiEnvReport {
    /// The queried variable name.
    pub variable: String,
    /// Files containing at least one definition or reference, sorted by path.
    pub files: Vec<CiEnvFile>,
    /// Non-fatal load/parse warnings.
    pub warnings: Vec<CiWarning>,
}

/// A workflow file with matching locations.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CiEnvFile {
    /// Repo-relative, slash-normalized path.
    pub path: String,
    /// Matching locations, sorted deterministically.
    pub locations: Vec<CiEnvLocation>,
}

/// A single definition or reference of the variable.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CiEnvLocation {
    /// Whether the variable is defined or referenced here.
    pub kind: EnvLocationKind,
    /// The scope the location lives in.
    pub scope: EnvScope,
    /// Owning job id for `job`/`step` scopes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job: Option<String>,
    /// The defined value (definitions only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Whether a location defines or references the variable.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum EnvLocationKind {
    Definition,
    Reference,
}

/// The structural scope of a location.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum EnvScope {
    Workflow,
    Job,
    Step,
}

/// Analyze all workflows under `ci.workflow_dirs` for `var`.
pub fn analyze_env(root: &Path, ci: &CiConfig, var: &str) -> CiEnvReport {
    let parsed = ParsedWorkflowSet::load(root, ci);
    analyze_env_from_parsed(&parsed, var)
}

/// Analyze CI environment usage with a request-scoped visibility snapshot.
#[doc(hidden)]
pub fn analyze_env_from_snapshot(
    root: &Path,
    ci: &CiConfig,
    var: &str,
    snapshot: &VisiblePathSnapshot,
) -> CiEnvReport {
    let parsed = ParsedWorkflowSet::load_from_snapshot(root, ci, snapshot);
    analyze_env_from_parsed(&parsed, var)
}

/// Analyze environment usage from the shared parsed workflow documents.
pub fn analyze_env_from_parsed(parsed: &ParsedWorkflowSet, var: &str) -> CiEnvReport {
    let reference_re = reference_regex(var);
    let mut files = Vec::new();
    let mut warnings = Vec::new();

    for document in &parsed.documents {
        match &document.value {
            Ok(value) => {
                let locations = collect_locations(value, var, &reference_re);
                if !locations.is_empty() {
                    files.push(CiEnvFile {
                        path: document.path.clone(),
                        locations,
                    });
                }
            }
            Err(error) => warnings.push(CiWarning {
                path: document.path.clone(),
                message: match error.kind {
                    WorkflowDocumentErrorKind::Read => {
                        format!("could not read workflow file: {}", error.message)
                    }
                    WorkflowDocumentErrorKind::Parse => {
                        format!("could not parse workflow YAML: {}", error.message)
                    }
                },
            }),
        }
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    CiEnvReport {
        variable: var.to_string(),
        files,
        warnings,
    }
}

#[cfg(test)]
mod tests;
