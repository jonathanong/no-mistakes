use no_mistakes::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

mod preserved_roots;
mod project_reopen;
mod views;

use project_reopen::{explicit_reopened_roots, unresolved_typed_reopen_suffixes};

pub(crate) fn discover_check_file_views_from_snapshot(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
    unique_exports_enabled: bool,
    snapshot: &no_mistakes::codebase::ts_source::VisiblePathSnapshot,
) -> views::CheckFileViews {
    let root_files = Some(visible_file_paths(snapshot, root));
    views::discover_check_file_views_with_absolute_lookup(
        root,
        config,
        skip_directories,
        unique_exports_enabled,
        root_files,
        |base| Some(visible_file_paths(snapshot, base)),
    )
}

pub(crate) fn select_graph_files(
    views: views::CheckFileViews,
    needs_shared_facts: bool,
    graph_requires_full_file_universe: bool,
    playwright_facts_configured: bool,
    dynamic_import_rules: bool,
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let needs_full_graph_files = graph_requires_full_file_universe || playwright_facts_configured;
    if needs_full_graph_files {
        return (views.filesystem, views.graph);
    }
    if needs_shared_facts && dynamic_import_rules {
        return (views.filesystem.clone(), views.filesystem);
    }
    (views.filesystem, Vec::new())
}

fn visible_file_paths(
    snapshot: &no_mistakes::codebase::ts_source::VisiblePathSnapshot,
    root: &Path,
) -> Vec<PathBuf> {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(root);
    let sources = snapshot.source_store_for(&root);
    sources.inventory().target_file_paths()
}

fn unique_exports_project_roots_with_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    inferred_roots: &mut no_mistakes::codebase::config::InferredRoots,
) -> Vec<PathBuf> {
    let rule_id = no_mistakes::codebase::unique_exports::RULE_ID;
    let mut roots = Vec::new();
    for rule in config.rule_applications(rule_id) {
        if rule.applies_to_repository() {
            roots.push(root.to_path_buf());
        }
        for project in rule
            .projects
            .iter()
            .filter_map(|project_name| config.projects.get(project_name))
        {
            if let Some(project_root) = project_root(root, project, inferred_roots) {
                roots.push(project_root);
            }
        }
    }
    let mut roots: Vec<_> = roots
        .into_iter()
        .map(|root| no_mistakes::codebase::ts_resolver::normalize_path(&root))
        .collect();
    roots.sort();
    roots.dedup();
    roots
}

fn preserved_project_roots_with_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    inferred_roots: &mut no_mistakes::codebase::config::InferredRoots,
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for rule in config.rules.iter().filter(|rule| rule.enabled) {
        let has_project_include = !rule.projects.is_empty() && !rule.include.is_empty();
        let preserves_project_root =
            rule.rule == no_mistakes::codebase::rules::FORBIDDEN_WORKSPACE_CLOSURE;
        if !has_project_include && !preserves_project_root {
            continue;
        }
        for project in rule
            .projects
            .iter()
            .filter_map(|project_name| config.projects.get(project_name))
        {
            if let Some(project_root) = project_root(root, project, inferred_roots) {
                roots.push(no_mistakes::codebase::ts_resolver::normalize_path(
                    &project_root,
                ));
            }
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

fn project_root(
    root: &Path,
    project: &no_mistakes::config::v2::schema::Project,
    inferred_roots: &mut no_mistakes::codebase::config::InferredRoots,
) -> Option<PathBuf> {
    if let Some(project_root) = project.root.as_deref() {
        return Some(root.join(project_root));
    }
    if project.type_ == Some(no_mistakes::config::v2::schema::ProjectType::Nextjs) {
        return inferred_roots.nextjs_root(root);
    }
    if project.type_ == Some(no_mistakes::config::v2::schema::ProjectType::Remix) {
        return inferred_roots.remix_root(root);
    }
    if project.type_ == Some(no_mistakes::config::v2::schema::ProjectType::Vitejs) {
        return inferred_roots.vitejs_root(root);
    }
    Some(root.to_path_buf())
}

#[cfg(test)]
mod tests;
