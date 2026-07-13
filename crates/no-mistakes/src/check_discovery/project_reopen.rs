use no_mistakes::config::v2::{schema::ProjectType, NoMistakesConfig};
use std::path::{Path, PathBuf};

pub(super) fn explicit_reopened_roots(
    root: &Path,
    config: &NoMistakesConfig,
    unique_exports_enabled: bool,
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for rule in config.rules.iter().filter(|rule| rule.enabled) {
        let reopens_project = rule.rule
            == no_mistakes::codebase::rules::FORBIDDEN_WORKSPACE_CLOSURE
            || (unique_exports_enabled
                && rule.rule == no_mistakes::codebase::unique_exports::RULE_ID)
            || !rule.include.is_empty();
        for project_name in &rule.projects {
            let Some(project_root) = config
                .projects
                .get(project_name)
                .and_then(|project| project.root.as_deref())
            else {
                continue;
            };
            let project_root =
                no_mistakes::codebase::ts_resolver::normalize_path(&root.join(project_root));
            if reopens_project && project_root.starts_with(root) && project_root != root {
                roots.push(project_root.clone());
            }
            for include in &rule.include {
                let Some(prefix) = super::preserved_roots::literal_include_prefix(include) else {
                    continue;
                };
                let include_root =
                    no_mistakes::codebase::ts_resolver::normalize_path(&project_root.join(prefix));
                if include_root.starts_with(root) && include_root != root {
                    roots.push(include_root);
                }
            }
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

pub(super) fn unresolved_typed_reopen_suffixes(config: &NoMistakesConfig) -> Vec<PathBuf> {
    let mut suffixes = Vec::new();
    for rule in config
        .rules
        .iter()
        .filter(|rule| rule.enabled && !rule.include.is_empty())
    {
        for project_name in &rule.projects {
            let Some(project) = config.projects.get(project_name) else {
                continue;
            };
            let rootless_typed = project.root.is_none()
                && matches!(
                    project.type_,
                    Some(ProjectType::Nextjs | ProjectType::Remix | ProjectType::Vitejs)
                );
            if !rootless_typed {
                continue;
            }
            for include in &rule.include {
                let Some(prefix) = super::preserved_roots::literal_include_prefix(include) else {
                    continue;
                };
                let mut accumulated = PathBuf::new();
                for component in prefix.components() {
                    accumulated.push(component);
                    let name = component.as_os_str().to_str().unwrap_or_default();
                    if no_mistakes::codebase::ts_source::is_skipped_dir(name) {
                        suffixes.push(accumulated.clone());
                    }
                }
            }
        }
    }
    suffixes.sort();
    suffixes.dedup();
    suffixes
}
