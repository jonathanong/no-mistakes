//! GitHub Actions workflow-graph analysis (`no-mistakes ci`).
//!
//! Parses `.github/workflows/*.{yml,yaml}` into a typed model and answers two
//! questions distinct from the TS module graph:
//!
//! - [`impact::analyze_impact`] — which workflows a changed file triggers, and
//!   each job's resolved permissions.
//! - [`env_query::analyze_env`] — where an environment variable is defined or
//!   referenced.
//!
//! Workflow directories come from [`CiConfig`], defaulting to
//! `.github/workflows`. Matching is deterministic and heuristic; see the
//! submodule docs for the documented limitations.

pub mod env_query;
pub mod impact;
pub mod model;
pub mod parse;
pub mod permissions;
pub mod triggers;

#[cfg(test)]
mod tests;

use crate::config::v2::schema::CiConfig;
use model::{CiWarning, Workflow};
use std::path::{Path, PathBuf};

pub use env_query::{analyze_env, CiEnvReport};
pub use impact::{analyze_impact, CiImpactReport};

/// A parsed set of workflows plus any non-fatal load warnings.
pub struct WorkflowSet {
    /// Parsed workflows, sorted by path.
    pub workflows: Vec<Workflow>,
    /// Non-fatal load/parse warnings.
    pub warnings: Vec<CiWarning>,
}

impl WorkflowSet {
    /// Discover and parse every workflow under `ci.workflow_dirs`.
    pub fn load(root: &Path, ci: &CiConfig) -> WorkflowSet {
        let mut workflows = Vec::new();
        let mut warnings = Vec::new();

        for path in discover_workflow_files(root, ci) {
            let rel = relative_slash(root, &path);
            // An unreadable discovered file yields empty content, which parses to
            // an empty workflow and contributes nothing — no special branch needed.
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            match parse::parse_workflow(&content, &rel) {
                Ok(workflow) => {
                    for note in &workflow.warnings {
                        warnings.push(CiWarning {
                            path: rel.clone(),
                            message: note.clone(),
                        });
                    }
                    workflows.push(workflow);
                }
                Err(error) => warnings.push(CiWarning {
                    path: rel,
                    message: format!("could not parse workflow YAML: {error}"),
                }),
            }
        }

        workflows.sort_by(|a, b| a.path.cmp(&b.path));
        WorkflowSet {
            workflows,
            warnings,
        }
    }
}

/// List workflow YAML files under the configured directories, sorted by path.
pub fn discover_workflow_files(root: &Path, ci: &CiConfig) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in &ci.workflow_dirs {
        let abs = root.join(dir);
        let Ok(entries) = std::fs::read_dir(&abs) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && has_yaml_extension(&path) {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn has_yaml_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml"))
}

/// Repo-relative, slash-normalized path for display.
pub fn relative_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
