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

use crate::codebase::ts_source::VisiblePathSnapshot;
use crate::config::v2::schema::CiConfig;
use model::{CiWarning, Workflow};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub use env_query::{analyze_env, analyze_env_from_snapshot, CiEnvReport};
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
        let snapshot = VisiblePathSnapshot::new(root);
        Self::load_from_snapshot(root, ci, &snapshot)
    }

    /// Parse workflows while reusing a request-scoped visibility snapshot.
    #[doc(hidden)]
    pub fn load_from_snapshot(
        root: &Path,
        ci: &CiConfig,
        snapshot: &VisiblePathSnapshot,
    ) -> WorkflowSet {
        let mut workflows = Vec::new();
        let mut warnings = Vec::new();

        for path in discover_workflow_files_from_snapshot(root, ci, snapshot) {
            let rel = relative_slash(root, &path);
            // Distinguish I/O failures (permissions, encoding) from parse errors
            // so the warning points at the real cause.
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
    let snapshot = VisiblePathSnapshot::new(root);
    discover_workflow_files_from_snapshot(root, ci, &snapshot)
}

#[doc(hidden)]
pub fn discover_workflow_files_from_snapshot(
    root: &Path,
    ci: &CiConfig,
    snapshot: &VisiblePathSnapshot,
) -> Vec<PathBuf> {
    let mut files = BTreeSet::new();
    let workflow_dirs: BTreeSet<PathBuf> = ci
        .workflow_dirs
        .iter()
        .map(|dir| crate::codebase::ts_resolver::normalize_path(&root.join(dir)))
        .collect();
    for abs in workflow_dirs {
        // Discover from the configured directory itself. Besides limiting each
        // snapshot to the files this analyzer needs, this crosses nested Git
        // worktree/submodule boundaries and supports directories outside root.
        let visible_paths = snapshot.paths_for(&abs);
        for path in visible_paths.iter() {
            let Some(direct_child) = path
                .strip_prefix(&abs)
                .ok()
                .and_then(|relative| relative.components().next())
                .map(|component| abs.join(component.as_os_str()))
            else {
                continue;
            };
            // Match on the direct child's extension only (not `is_file`), so a
            // visible descendant of a directory mistakenly named like a
            // workflow still surfaces a clear read warning.
            if has_yaml_extension(&direct_child) {
                files.insert(direct_child);
            }
        }
    }
    files.into_iter().collect()
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
