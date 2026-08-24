use super::actions::{action_descriptors_for_path, action_job_users};
use super::job_selection::{entry_change_is_global, entry_changed_jobs};
use super::project::{project_impact, ImpactInputs};
use super::reachability::reachable_actions;
use super::snapshot::{changed_paths, topology_from_tree};
use super::yaml::normalize_entry;
use super::CiTopologyImpactReport;
use anyhow::{anyhow, Context, Result};
use git2::Repository;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn topology_impact_report(
    root: &Path,
    base_revision: &str,
    head_revision: &str,
    entry_workflow: &str,
) -> Result<CiTopologyImpactReport> {
    let cwd = std::env::current_dir().context("read current directory")?;
    let root = crate::codebase::ts_resolver::normalize_path(&crate::cli::resolve_optional_root(
        Some(root),
        &cwd,
    ));
    let repo = Repository::discover(&root).context("open repository for ci topology impact")?;
    let workdir = repo
        .workdir()
        .context("bare repositories are not supported")?;
    let canonical_root =
        std::fs::canonicalize(&root).context("canonicalize ci topology impact root")?;
    let canonical_workdir =
        std::fs::canonicalize(workdir).context("canonicalize repository worktree root")?;
    if canonical_root != canonical_workdir {
        return Err(anyhow!("root must be the repository worktree root"));
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
    let action_jobs = action_job_users(
        &repo,
        &base_tree,
        &head_tree,
        &reachable_actions,
        &changed_actions,
        &base_topology,
        &head_topology,
    );
    let unowned_action = changed_paths.iter().any(|path| {
        reachable_actions.iter().any(|action| {
            (action.is_empty() || path == action || path.starts_with(&format!("{action}/")))
                && action_descriptors_for_path(&base_tree, &head_tree, path).is_empty()
        })
    });
    let entry_global_change = changed_paths.iter().any(|path| path == &entry)
        && entry_change_is_global(&repo, &base_tree, &head_tree, &entry);
    let changed_entry_jobs = entry_changed_jobs(&repo, &base_tree, &head_tree, &entry);
    Ok(project_impact(ImpactInputs {
        base_revision: base.id().to_string(),
        head_revision: head.id().to_string(),
        changed_paths,
        entry_workflow,
        base: &base_topology,
        head: &head_topology,
        reachable_actions: &reachable_actions,
        changed_actions: &changed_actions,
        action_jobs: &action_jobs,
        changed_entry_jobs: &changed_entry_jobs,
        entry_global_change,
        unowned_action,
    }))
}
