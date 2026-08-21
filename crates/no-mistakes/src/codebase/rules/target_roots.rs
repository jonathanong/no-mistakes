use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) fn target_roots(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
) -> Vec<PathBuf> {
    let mut inferred_roots = crate::codebase::config::InferredRoots::default();
    target_roots_with_inferred(root, config, rule, &mut inferred_roots)
}

pub(crate) fn target_roots_with_inferred(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
    inferred_roots: &mut crate::codebase::config::InferredRoots,
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if rule.applies_to_repository() {
        roots.push(root.to_path_buf());
    }
    for project_name in &rule.projects {
        let Some(project) = config.projects.get(project_name) else {
            continue;
        };
        if let Some(project_root) = target_project_root(root, project, inferred_roots) {
            roots.push(project_root);
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

pub(crate) fn file_allowed_by_roots_and_skip(
    root: &Path,
    skip: &HashSet<&str>,
    path: &Path,
    roots: &[PathBuf],
) -> bool {
    let mut matching_roots = roots.iter().filter(|rule_root| path.starts_with(rule_root));
    let Some(first_root) = matching_roots.next() else {
        return false;
    };

    if !crate::codebase::ts_source::is_under_skipped_dir(root, path, skip) {
        return true;
    }

    if !crate::codebase::ts_source::is_under_skipped_dir(first_root, path, skip) {
        return true;
    }

    matching_roots
        .any(|rule_root| !crate::codebase::ts_source::is_under_skipped_dir(rule_root, path, skip))
}

pub(crate) fn skip_dir_set(config: &crate::config::v2::NoMistakesConfig) -> HashSet<&str> {
    config
        .filesystem
        .skip_directories
        .iter()
        .map(String::as_str)
        .collect()
}

pub(crate) fn target_project_root(
    root: &Path,
    project: &crate::config::v2::schema::Project,
    inferred_roots: &mut crate::codebase::config::InferredRoots,
) -> Option<PathBuf> {
    if let Some(project_root) = project.root.as_deref() {
        return Some(root.join(project_root));
    }
    match project.type_ {
        Some(crate::config::v2::schema::ProjectType::Nextjs) => inferred_roots.nextjs_root(root),
        Some(crate::config::v2::schema::ProjectType::Remix) => inferred_roots.remix_root(root),
        Some(crate::config::v2::schema::ProjectType::Vitejs) => inferred_roots.vitejs_root(root),
        _ => Some(root.to_path_buf()),
    }
}
