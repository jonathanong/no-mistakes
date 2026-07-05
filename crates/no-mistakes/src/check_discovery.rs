use no_mistakes::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

pub(crate) fn discover_check_files(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
    unique_exports_enabled: bool,
) -> Vec<PathBuf> {
    let preserved_roots = include_preserved_roots(root, config, skip_directories);
    let mut files = no_mistakes::codebase::ts_source::discover_files_preserving_roots(
        root,
        skip_directories,
        &preserved_roots,
    );
    if unique_exports_enabled {
        for project_root in unique_exports_project_roots(root, config) {
            if project_root == root {
                continue;
            }
            files.extend(no_mistakes::codebase::ts_source::discover_files(
                &project_root,
                skip_directories,
            ));
        }
    }
    files.sort();
    files.dedup();
    files
}

fn include_preserved_roots(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let mut inferred_roots = no_mistakes::codebase::config::InferredRoots::default();
    for rule in config.rules.iter().filter(|rule| rule.enabled) {
        if rule.rule == no_mistakes::codebase::rules::FORBIDDEN_WORKSPACE_CLOSURE {
            for project_name in &rule.projects {
                let Some(project) = config.projects.get(project_name) else {
                    continue;
                };
                if let Some(project_root) = project_root(root, project, &mut inferred_roots) {
                    roots.push(project_root);
                }
            }
        }
        for include in &rule.include {
            push_include_preserved_roots(&mut roots, root, include, skip_directories);
            for project_name in &rule.projects {
                let Some(project) = config.projects.get(project_name) else {
                    continue;
                };
                if let Some(project_root) = project_root(root, project, &mut inferred_roots) {
                    push_include_preserved_roots(
                        &mut roots,
                        &project_root,
                        include,
                        skip_directories,
                    );
                }
            }
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

fn push_include_preserved_roots(
    roots: &mut Vec<PathBuf>,
    base: &Path,
    include: &str,
    skip_directories: &[String],
) {
    if let Some(prefix) = literal_include_prefix(include) {
        roots.push(base.join(&prefix));
    }
    if let Some(suffix) = leading_globstar_literal_prefix(include) {
        roots.extend(descendant_dirs_matching_suffix(
            base,
            &suffix,
            skip_directories,
        ));
    }
}

fn descendant_dirs_matching_suffix(
    base: &Path,
    suffix: &Path,
    skip_directories: &[String],
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    collect_descendant_dirs_matching_suffix(base, base, suffix, skip_directories, &mut roots);
    roots
}

fn collect_descendant_dirs_matching_suffix(
    base: &Path,
    dir: &Path,
    suffix: &Path,
    skip_directories: &[String],
    roots: &mut Vec<PathBuf>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !entry
            .file_type()
            .ok()
            .is_some_and(|file_type| file_type.is_dir())
        {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_str().unwrap_or_default();
        let skipped = no_mistakes::codebase::ts_source::is_skipped_dir(name)
            || skip_directories.iter().any(|skip| skip == name);
        if path
            .strip_prefix(base)
            .ok()
            .is_some_and(|rel| rel.ends_with(suffix))
        {
            roots.push(path.clone());
        }
        if skipped {
            continue;
        }
        collect_descendant_dirs_matching_suffix(base, &path, suffix, skip_directories, roots);
    }
}

fn leading_globstar_literal_prefix(include: &str) -> Option<PathBuf> {
    include.strip_prefix("**/").and_then(literal_include_prefix)
}

fn literal_include_prefix(include: &str) -> Option<PathBuf> {
    let prefix = include
        .split(['*', '?', '[', '{'])
        .next()
        .unwrap_or_default()
        .trim_end_matches('/');
    if prefix.is_empty() {
        return None;
    }
    Some(PathBuf::from(prefix))
}

fn unique_exports_project_roots(root: &Path, config: &NoMistakesConfig) -> Vec<PathBuf> {
    let rule_id = no_mistakes::codebase::unique_exports::RULE_ID;
    let mut roots = Vec::new();
    let mut inferred_roots = no_mistakes::codebase::config::InferredRoots::default();
    for rule in config.rule_applications(rule_id) {
        if rule.applies_to_repository() {
            roots.push(root.to_path_buf());
        }
        for project_name in &rule.projects {
            let Some(project) = config.projects.get(project_name) else {
                continue;
            };
            if let Some(project_root) = project_root(root, project, &mut inferred_roots) {
                roots.push(project_root);
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
        return inferred_roots
            .nextjs
            .get_or_insert_with(|| no_mistakes::codebase::config::infer_nextjs_root(root))
            .clone();
    }
    if project.type_ == Some(no_mistakes::config::v2::schema::ProjectType::Remix) {
        return inferred_roots
            .remix
            .get_or_insert_with(|| no_mistakes::codebase::config::infer_remix_root(root))
            .clone();
    }
    if project.type_ == Some(no_mistakes::config::v2::schema::ProjectType::Vitejs) {
        return inferred_roots
            .vitejs
            .get_or_insert_with(|| no_mistakes::codebase::config::infer_vitejs_root(root))
            .clone();
    }
    Some(root.to_path_buf())
}

#[cfg(test)]
mod tests;
