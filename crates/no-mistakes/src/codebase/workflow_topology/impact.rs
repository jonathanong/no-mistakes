//! Revision-aware impact projection over the stable workflow-topology graph.
//!
//! The topology schema remains version 1.  This module intentionally exposes
//! a separate, versioned report because callers need base/head provenance and
//! fail-open diagnostics that do not belong in a snapshot graph.

use super::load_workflow_topology_from_parsed;
use super::model::{WorkflowTopology, WorkflowTopologyDiagnostic};
use crate::codebase::ci_workflows::{
    ParsedWorkflowDocument, ParsedWorkflowSet, WorkflowDocumentError,
};
use crate::config::v2::schema::CiConfig;
use anyhow::{anyhow, Context, Result};
use git2::{Delta, Repository, Tree};
use serde::Serialize;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

pub const CI_TOPOLOGY_IMPACT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CiTopologyImpactDiagnostic {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CiTopologyImpactReport {
    pub schema_version: u32,
    pub base_revision: String,
    pub head_revision: String,
    pub changed_paths: Vec<String>,
    pub affected_workflows: Vec<String>,
    pub affected_root_job_ids: Vec<String>,
    pub diagnostics: Vec<CiTopologyImpactDiagnostic>,
    pub global_fallback: bool,
}

pub fn topology_impact_report(
    root: &Path,
    base_revision: &str,
    head_revision: &str,
    entry_workflow: &str,
) -> Result<CiTopologyImpactReport> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let repo = Repository::discover(&root).context("open repository for ci topology impact")?;
    let workdir = repo
        .workdir()
        .context("bare repositories are not supported")?;
    if !root.starts_with(workdir) && !workdir.starts_with(&root) {
        return Err(anyhow!("root must be inside the repository worktree"));
    }
    let base = repo
        .revparse_single(base_revision)
        .context("resolve base revision")?
        .peel_to_commit()?;
    let head = repo
        .revparse_single(head_revision)
        .context("resolve head revision")?
        .peel_to_commit()?;
    let base_tree = base.tree()?;
    let head_tree = head.tree()?;
    let changed_paths = changed_paths(&repo, &base_tree, &head_tree)?;
    let base_topology = topology_from_tree(&repo, &base_tree)?;
    let head_topology = topology_from_tree(&repo, &head_tree)?;
    let entry = normalize_entry(entry_workflow);
    let reachable_actions = reachable_actions(&repo, &base_tree, &head_tree, &entry);
    let changed_actions = changed_paths
        .iter()
        .flat_map(|path| action_descriptors_for_path(&base_tree, &head_tree, path))
        .collect::<BTreeSet<_>>();
    let entry_global_change = changed_paths.iter().any(|path| path == &entry)
        && entry_change_is_global(&repo, &base_tree, &head_tree, &entry);
    Ok(project_impact(
        base.id().to_string(),
        head.id().to_string(),
        changed_paths,
        entry_workflow,
        &base_topology,
        &head_topology,
        &reachable_actions,
        &changed_actions,
        entry_global_change,
    ))
}

fn changed_paths(repo: &Repository, base: &Tree<'_>, head: &Tree<'_>) -> Result<Vec<String>> {
    let diff = repo.diff_tree_to_tree(Some(base), Some(head), None)?;
    let mut paths = BTreeSet::new();
    for delta in diff.deltas() {
        for file in [delta.old_file(), delta.new_file()] {
            if let Some(path) = file.path() {
                paths.insert(path.to_string_lossy().replace('\\', "/"));
            }
        }
        if delta.status() == Delta::Unmodified {
            continue;
        }
    }
    Ok(paths.into_iter().collect())
}

fn topology_from_tree(repo: &Repository, tree: &Tree<'_>) -> Result<WorkflowTopology> {
    let mut documents = Vec::new();
    collect_workflows(repo, tree, "", &mut documents)?;
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    let parsed = ParsedWorkflowSet { documents };
    Ok(load_workflow_topology_from_parsed(
        Path::new("."),
        &CiConfig::default(),
        &parsed,
        &[],
    ))
}

fn collect_workflows(
    repo: &Repository,
    tree: &Tree<'_>,
    prefix: &str,
    documents: &mut Vec<ParsedWorkflowDocument>,
) -> Result<()> {
    for entry in tree {
        let name = entry.name().unwrap_or_default();
        let path = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{prefix}/{name}")
        };
        if entry.kind() == Some(git2::ObjectType::Tree) {
            if ".github" == name || prefix.starts_with(".github") {
                collect_workflows(
                    repo,
                    &entry.to_object(repo)?.peel_to_tree()?,
                    &path,
                    documents,
                )?;
            }
            continue;
        }
        if !path.starts_with(".github/workflows/")
            || !(path.ends_with(".yml") || path.ends_with(".yaml"))
        {
            continue;
        }
        let blob = entry.to_object(repo)?.peel_to_blob()?;
        let value = serde_yaml::from_slice(blob.content()).map_err(|error| WorkflowDocumentError {
            kind: crate::codebase::ci_workflows::WorkflowDocumentErrorKind::Parse,
            message: error.to_string(),
        });
        documents.push(ParsedWorkflowDocument { path, value });
    }
    Ok(())
}

fn project_impact(
    base_revision: String,
    head_revision: String,
    changed_paths: Vec<String>,
    entry_workflow: &str,
    base: &WorkflowTopology,
    head: &WorkflowTopology,
    reachable_actions: &BTreeSet<String>,
    changed_actions: &BTreeSet<String>,
    entry_global_change: bool,
) -> CiTopologyImpactReport {
    let mut diagnostics = Vec::new();
    let mut global_fallback = false;
    let entry = normalize_entry(entry_workflow);
    let has_entry = base
        .workflows
        .iter()
        .chain(&head.workflows)
        .any(|workflow| workflow.path == entry);
    if !has_entry {
        global_fallback = true;
        diagnostics.push(CiTopologyImpactDiagnostic {
            code: "missing-entry-workflow".into(),
            message: format!("entry workflow {entry} is absent from both revisions"),
            workflow_path: Some(entry.clone()),
        });
    }
    let reachable = reachable_workflows(&entry, base, head);
    for diagnostic in base.diagnostics.iter().chain(&head.diagnostics) {
        // A malformed document and an endpoint outside the entry closure are
        // inherently unbounded. Other graph diagnostics remain auditable but
        // are local when every endpoint belongs to this root closure.
        if diagnostic.code.as_str() == "malformed-workflow"
            || !reachable.contains(&diagnostic.workflow_path)
        {
            global_fallback = true;
        }
        diagnostics.push(topology_diagnostic(diagnostic));
    }
    // Any changed workflow/action that is reachable from the entry can affect
    // every root job; graph construction already detects malformed references.
    let impacted = changed_paths
        .iter()
        .any(|path| reachable.contains(path) || !reachable_actions.is_disjoint(changed_actions));
    global_fallback |= entry_global_change;
    let topology = head;
    let mut jobs: BTreeSet<String> = if impacted || global_fallback {
        topology
            .jobs
            .iter()
            .filter(|job| job.workflow_id == entry)
            .map(|job| job.id.clone())
            .collect()
    } else {
        BTreeSet::new()
    };
    add_needs_prerequisites(&mut jobs, topology);
    let affected_workflows = if impacted || global_fallback {
        reachable.into_iter().collect()
    } else {
        Vec::new()
    };
    diagnostics.sort_by(|left, right| {
        (
            left.code.as_str(),
            left.workflow_path.as_deref(),
            left.message.as_str(),
        )
            .cmp(&(
                right.code.as_str(),
                right.workflow_path.as_deref(),
                right.message.as_str(),
            ))
    });
    diagnostics.dedup_by(|left, right| {
        left.code == right.code
            && left.workflow_path == right.workflow_path
            && left.message == right.message
    });
    CiTopologyImpactReport {
        schema_version: CI_TOPOLOGY_IMPACT_SCHEMA_VERSION,
        base_revision,
        head_revision,
        changed_paths,
        affected_workflows,
        affected_root_job_ids: jobs.into_iter().collect(),
        diagnostics,
        global_fallback,
    }
}

fn normalize_entry(entry: &str) -> String {
    if entry.contains('/') {
        entry.trim_start_matches("./").to_string()
    } else {
        format!(".github/workflows/{entry}")
    }
}

fn topology_diagnostic(diagnostic: &WorkflowTopologyDiagnostic) -> CiTopologyImpactDiagnostic {
    CiTopologyImpactDiagnostic {
        code: diagnostic.code.as_str().to_string(),
        message: diagnostic.message.clone(),
        workflow_path: Some(diagnostic.workflow_path.clone()),
    }
}

fn entry_change_is_global(
    repo: &Repository,
    base: &Tree<'_>,
    head: &Tree<'_>,
    entry: &str,
) -> bool {
    let Some(base) = yaml_at(repo, base, entry) else {
        return true;
    };
    let Some(head) = yaml_at(repo, head, entry) else {
        return true;
    };
    let (Some(base), Some(head)) = (base.as_mapping(), head.as_mapping()) else {
        return true;
    };
    let jobs = serde_yaml::Value::String("jobs".into());
    let mut base = base.clone();
    let mut head = head.clone();
    base.remove(&jobs);
    head.remove(&jobs);
    base != head
}

fn yaml_at(repo: &Repository, tree: &Tree<'_>, path: &str) -> Option<serde_yaml::Value> {
    let entry = tree.get_path(Path::new(path)).ok()?;
    let blob = entry.to_object(repo).ok()?.peel_to_blob().ok()?;
    serde_yaml::from_slice(blob.content()).ok()
}

fn action_descriptors_for_path(base: &Tree<'_>, head: &Tree<'_>, path: &str) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    let mut cursor = Path::new(path).parent().map(Path::to_path_buf);
    while let Some(directory_path) = cursor {
        let next = directory_path.parent().map(Path::to_path_buf);
        let directory = directory_path.to_string_lossy().replace('\\', "/");
        if !directory.starts_with(".github/actions/") {
            break;
        }
        if [base, head].iter().any(|tree| {
            ["action.yml", "action.yaml"].iter().any(|name| {
                tree.get_path(Path::new(&format!("{directory}/{name}")))
                    .is_ok()
            })
        }) {
            result.insert(directory);
            break;
        }
        cursor = next;
    }
    result
}

fn reachable_actions(
    repo: &Repository,
    base: &Tree<'_>,
    head: &Tree<'_>,
    entry: &str,
) -> BTreeSet<String> {
    let mut visited = BTreeSet::new();
    for tree in [base, head] {
        let Some(value) = yaml_at(repo, tree, entry) else {
            continue;
        };
        collect_uses(&value, tree, repo, &mut visited);
    }
    visited
}

fn collect_uses(
    value: &serde_yaml::Value,
    tree: &Tree<'_>,
    repo: &Repository,
    visited: &mut BTreeSet<String>,
) {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (key, value) in map {
                if key.as_str() == Some("uses") {
                    if let Some(uses) = value.as_str().filter(|uses| uses.starts_with("./")) {
                        let action = uses
                            .trim_start_matches("./")
                            .trim_end_matches('/')
                            .to_string();
                        if visited.insert(action.clone()) {
                            for descriptor in [
                                format!("{action}/action.yml"),
                                format!("{action}/action.yaml"),
                            ] {
                                if let Some(action_yaml) = yaml_at(repo, tree, &descriptor) {
                                    collect_uses(&action_yaml, tree, repo, visited);
                                }
                            }
                        }
                    }
                }
                collect_uses(value, tree, repo, visited);
            }
        }
        serde_yaml::Value::Sequence(values) => {
            for value in values {
                collect_uses(value, tree, repo, visited);
            }
        }
        _ => {}
    }
}

fn reachable_workflows(
    entry: &str,
    base: &WorkflowTopology,
    head: &WorkflowTopology,
) -> BTreeSet<String> {
    let mut calls = HashMap::<String, Vec<String>>::new();
    let mut job_workflow = HashMap::<String, String>::new();
    for topology in [base, head] {
        for job in &topology.jobs {
            job_workflow.insert(job.id.clone(), job.workflow_id.clone());
        }
        for edge in &topology.edges {
            if let super::model::WorkflowTopologyEdge::Calls(call) = edge {
                if let Some(target) = &call.to {
                    if let Some(source) = job_workflow.get(&call.from) {
                        calls
                            .entry(source.clone())
                            .or_default()
                            .push(target.clone());
                    }
                }
            }
        }
    }
    let mut seen = BTreeSet::new();
    let mut pending = vec![entry.to_string()];
    while let Some(path) = pending.pop() {
        if seen.insert(path.clone()) {
            pending.extend(calls.remove(&path).unwrap_or_default());
        }
    }
    seen
}

fn add_needs_prerequisites(selected: &mut BTreeSet<String>, topology: &WorkflowTopology) {
    let mut dependencies = HashMap::<String, Vec<String>>::new();
    for edge in &topology.edges {
        if let super::model::WorkflowTopologyEdge::Needs(edge) = edge {
            dependencies
                .entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
        }
    }
    let mut pending: Vec<String> = selected.iter().cloned().collect();
    while let Some(job) = pending.pop() {
        for dependency in dependencies.remove(&job).unwrap_or_default() {
            if selected.insert(dependency.clone()) {
                pending.push(dependency);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_entry;

    #[test]
    fn normalizes_entry_workflow_basenames_without_rewriting_paths() {
        assert_eq!(normalize_entry("ci.yml"), ".github/workflows/ci.yml");
        assert_eq!(
            normalize_entry("./.github/workflows/ci.yml"),
            ".github/workflows/ci.yml"
        );
    }
}
