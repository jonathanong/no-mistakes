use crate::codebase::ts_resolver::normalize_path;
use crate::config::v2::schema::{NoMistakesConfig, ProjectType};
use std::path::{Path, PathBuf};

pub(super) fn project_roots(root: &Path, config: &NoMistakesConfig) -> Vec<PathBuf> {
    let mut inferred = crate::codebase::config::InferredRoots::default();
    let mut roots = Vec::new();
    for project in config.projects.values() {
        if project.type_ != Some(ProjectType::Nextjs) {
            continue;
        }
        let Some(project_root) =
            crate::codebase::rules::target_roots::target_project_root(root, project, &mut inferred)
        else {
            continue;
        };
        roots.push(normalize_path(&project_root));
    }
    roots.sort();
    roots.dedup();
    roots
}

pub(super) fn consumed_export(file: &Path, roots: &[PathBuf], name: &str) -> bool {
    roots.iter().any(|project_root| {
        crate::codebase::unique_exports::is_framework_entrypoint_file(file, project_root)
            && crate::codebase::unique_exports::framework_consumed_export(file, project_root, name)
    })
}
