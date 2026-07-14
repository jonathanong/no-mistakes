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
use super::{discover_workflow_files, discover_workflow_files_from_snapshot, relative_slash};
use crate::codebase::ts_source::VisiblePathSnapshot;
use crate::config::v2::schema::CiConfig;
use serde::Serialize;
use serde_yaml::Value;
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
    analyze_env_files(root, var, discover_workflow_files(root, ci))
}

/// Analyze CI environment usage with a request-scoped visibility snapshot.
#[doc(hidden)]
pub fn analyze_env_from_snapshot(
    root: &Path,
    ci: &CiConfig,
    var: &str,
    snapshot: &VisiblePathSnapshot,
) -> CiEnvReport {
    analyze_env_files(
        root,
        var,
        discover_workflow_files_from_snapshot(root, ci, snapshot),
    )
}

fn analyze_env_files(
    root: &Path,
    var: &str,
    workflow_files: Vec<std::path::PathBuf>,
) -> CiEnvReport {
    let reference_re = reference_regex(var);
    let mut files = Vec::new();
    let mut warnings = Vec::new();

    for path in workflow_files {
        let rel = relative_slash(root, &path);
        // Distinguish I/O failures from parse errors so the warning is accurate.
        let content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) => {
                warnings.push(CiWarning {
                    path: rel,
                    message: format!("could not read workflow file: {error}"),
                });
                continue;
            }
        };
        let value: Value = match serde_yaml::from_str(&content) {
            Ok(value) => value,
            Err(error) => {
                warnings.push(CiWarning {
                    path: rel,
                    message: format!("could not parse workflow YAML: {error}"),
                });
                continue;
            }
        };
        let locations = collect_locations(&value, var, &reference_re);
        if !locations.is_empty() {
            files.push(CiEnvFile {
                path: rel,
                locations,
            });
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
