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
    let root_files = Some(relative_visible_paths(snapshot, root));
    views::discover_check_file_views_with_external_lookup(
        root,
        config,
        skip_directories,
        unique_exports_enabled,
        root_files,
        |base| Some(relative_visible_paths(snapshot, base)),
    )
}

fn relative_visible_paths(
    snapshot: &no_mistakes::codebase::ts_source::VisiblePathSnapshot,
    root: &Path,
) -> Vec<String> {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(root);
    snapshot
        .paths_for(&root)
        .iter()
        .filter_map(|path| {
            if let Ok(relative) = path.strip_prefix(&root) {
                return Some(relative.to_string_lossy().into_owned());
            }
            no_mistakes::codebase::ts_resolver::normalize_path(path)
                .strip_prefix(&root)
                .ok()
                .map(|relative| relative.to_string_lossy().into_owned())
        })
        .collect()
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
