//! `--workflow` filter normalization, selection, and diagnostics, split out
//! of [`super`] to stay under the crate's per-file line limit. Re-exported
//! by [`super`] so every `topology_graph::select_workflow_paths`-style path
//! elsewhere in the crate keeps working unchanged.

use super::super::model;
use super::super::posix_path;
use super::super::topology_identifiers;
use std::collections::{HashMap, HashSet};

pub const WORKFLOWS_DIRECTORY: &str = ".github/workflows";

/// Every workflow when `requested` is empty; otherwise the normalized
/// requested set plus, for each requested workflow that exists, its
/// transitive local reusable-workflow callees.
///
/// `workflow_dirs` is the repository's configured `ci.workflowDirs` (the
/// same list [`super::super::mod`]'s discovery pass uses) — a bare
/// basename filter is resolved against it rather than the vendored
/// engine's hardcoded default, so `--workflow` stays correct when a repo
/// configures a non-default workflow directory.
pub fn select_workflow_paths(
    requested: &[String],
    workflows: &HashMap<String, model::WorkflowNode>,
    edges: &[model::WorkflowTopologyEdge],
    workflow_dirs: &[String],
) -> HashSet<String> {
    if requested.is_empty() {
        return workflows.keys().cloned().collect();
    }
    let mut selected: HashSet<String> = requested
        .iter()
        .filter_map(|path| normalize_workflow_filter(path, workflow_dirs, workflows))
        .collect();
    let initially_selected: Vec<String> = selected.iter().cloned().collect();

    let mut call_adjacency: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        if let model::WorkflowTopologyEdge::Calls(call) = edge {
            if call.local {
                if let Some(to) = &call.to {
                    let from = topology_identifiers::workflow_path_from_id(&call.from).to_string();
                    call_adjacency.entry(from).or_default().push(to.clone());
                }
            }
        }
    }

    for path in initially_selected {
        if !workflows.contains_key(&path) {
            continue;
        }
        selected.extend(transitive_local_callees(&path, &call_adjacency));
    }
    selected
}

/// Transitive local-call closure starting from (but excluding) `start`,
/// including when a call cycle leads back to `start` itself.
fn transitive_local_callees(
    start: &str,
    adjacency: &HashMap<String, Vec<String>>,
) -> HashSet<String> {
    let mut result = HashSet::new();
    let mut seen: HashSet<String> = HashSet::from([start.to_string()]);
    let mut stack = vec![start.to_string()];
    while let Some(current) = stack.pop() {
        let Some(targets) = adjacency.get(&current) else {
            continue;
        };
        for target in targets {
            if seen.insert(target.clone()) {
                result.insert(target.clone());
                stack.push(target.clone());
            }
        }
    }
    result
}

pub fn diagnose_workflow_filters(
    requested: &[String],
    workflows: &HashMap<String, model::WorkflowNode>,
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
    workflow_dirs: &[String],
) {
    for requested_path in requested {
        let Some(path) = normalize_workflow_filter(requested_path, workflow_dirs, workflows) else {
            diagnostics.push(model::WorkflowTopologyDiagnostic::new(
                model::DiagnosticCode::InvalidWorkflowFilter,
                format!(
                    "workflow filter must be a basename or a path inside {}: {requested_path}",
                    configured_dirs(workflow_dirs).join(", ")
                ),
                requested_path.clone(),
            ));
            continue;
        };
        if workflows.contains_key(&path) {
            continue;
        }
        diagnostics.push(model::WorkflowTopologyDiagnostic::new(
            model::DiagnosticCode::UnknownWorkflowFilter,
            format!("workflow filter does not match a repository workflow: {requested_path}"),
            path,
        ));
    }
}

/// The repo's configured workflow directories, falling back to the
/// vendored engine's single hardcoded default when none are configured
/// (an explicit empty `ci.workflowDirs: []` override — `CiConfig`'s own
/// default is always non-empty).
fn configured_dirs(workflow_dirs: &[String]) -> Vec<&str> {
    if workflow_dirs.is_empty() {
        vec![WORKFLOWS_DIRECTORY]
    } else {
        workflow_dirs.iter().map(String::as_str).collect()
    }
}

/// Normalizes a `--workflow` filter to a `<workflow-dir>/<file>` path, or
/// `None` when it's absolute, escapes every configured workflow directory
/// via `..`, or (once it contains a `/`) doesn't resolve to a direct child
/// of any of them. A bare basename is resolved against whichever
/// configured directory actually holds a matching discovered workflow,
/// falling back to the first configured directory when none do (so an
/// unresolvable filter still gets a deterministic path for the
/// `unknown-workflow-filter` diagnostic).
fn normalize_workflow_filter(
    path: &str,
    workflow_dirs: &[String],
    workflows: &HashMap<String, model::WorkflowNode>,
) -> Option<String> {
    let slashed = path.replace('\\', "/");
    let candidate = strip_leading_dot_slashes(&slashed);
    if candidate.is_empty()
        || candidate.starts_with('/')
        || candidate.split('/').any(|segment| segment == "..")
    {
        return None;
    }
    let dirs = configured_dirs(workflow_dirs);
    if !candidate.contains('/') {
        if candidate == "." || candidate == ".." {
            return None;
        }
        return Some(
            dirs.iter()
                .map(|dir| format!("{dir}/{candidate}"))
                .find(|joined| workflows.contains_key(joined))
                .unwrap_or_else(|| format!("{}/{candidate}", dirs[0])),
        );
    }
    let normalized = posix_path::normalize(candidate);
    let dirname = posix_path::dirname(&normalized);
    dirs.iter().any(|dir| *dir == dirname).then_some(normalized)
}

fn strip_leading_dot_slashes(path: &str) -> &str {
    let mut rest = path;
    while let Some(stripped) = rest.strip_prefix("./") {
        rest = stripped;
    }
    rest
}

/// `calls` edges belong to the selection whenever their origin does,
/// regardless of whether their (possibly absent, for remote calls)
/// destination is selected — the destination is only meaningfully checked
/// for the other three edge kinds.
pub fn edge_belongs_to_selection(
    edge: &model::WorkflowTopologyEdge,
    selected: &HashSet<String>,
) -> bool {
    if !selected.contains(topology_identifiers::workflow_path_from_id(edge.from())) {
        return false;
    }
    matches!(edge, model::WorkflowTopologyEdge::Calls(_))
        || selected.contains(topology_identifiers::workflow_path_from_id(
            edge.to().unwrap_or(""),
        ))
}
