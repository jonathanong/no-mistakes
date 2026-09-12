use super::super::load_workflow_topology_from_parsed;
use super::super::model::WorkflowTopology;
use crate::codebase::ci_workflows::{
    ParsedWorkflowDocument, ParsedWorkflowSet, WorkflowDocumentError,
};
use crate::config::v2::schema::CiConfig;
use anyhow::Result;
use git2::{Delta, Repository, Tree};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn changed_paths(
    repo: &Repository,
    base: &Tree<'_>,
    head: &Tree<'_>,
) -> Result<Vec<String>> {
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

pub(super) fn topology_from_tree(repo: &Repository, tree: &Tree<'_>) -> Result<WorkflowTopology> {
    let mut documents = Vec::new();
    collect_workflows(repo, tree, "", &mut documents)?;
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(load_workflow_topology_from_parsed(
        Path::new("."),
        &CiConfig::default(),
        &ParsedWorkflowSet { documents },
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
            if is_workflow_tree(&path) {
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

fn is_workflow_tree(path: &str) -> bool {
    path == ".github" || path == ".github/workflows" || path.starts_with(".github/workflows/")
}

#[cfg(test)]
#[path = "snapshot_tests.rs"]
mod tests;
