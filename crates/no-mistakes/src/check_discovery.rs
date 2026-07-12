use no_mistakes::config::v2::NoMistakesConfig;
use preserved_roots::include_preserved_roots;
use std::path::{Path, PathBuf};

mod preserved_roots;

/// Discovers files for `no-mistakes check`, optionally reusing a git-visible file
/// list a caller already fetched via `git_visible_files` instead of spawning
/// `git ls-files` again for `root`. Pass `git_files: None` to fetch it
/// internally. `no-mistakes check` calls this twice for the same `root` when
/// `forbidden-dependencies` is configured — once with the configured
/// skip-directory filter, once with none — so `check_runner.rs` fetches the
/// list once and passes it to both calls via `Some(&files)`.
///
/// The unique-exports loop below is intentionally NOT threaded through
/// `git_files`: it discovers each configured project root independently, and
/// those roots are frequently different directories than `root` (a nested
/// `web/`/`backend/` project, an inferred Next.js/Remix/Vite.js root, etc). A
/// git-visible list fetched for `root` is relative to `root`'s working
/// directory, so reusing it for a different project root would silently
/// mis-resolve paths; each project root keeps discovering its own list.
pub(crate) fn discover_check_files(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
    unique_exports_enabled: bool,
    git_files: Option<&[String]>,
) -> Vec<PathBuf> {
    let preserved_roots = include_preserved_roots(root, config, skip_directories);
    let mut files =
        no_mistakes::codebase::ts_source::discover_files_preserving_roots_from_git_files(
            root,
            skip_directories,
            &preserved_roots,
            git_files,
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
