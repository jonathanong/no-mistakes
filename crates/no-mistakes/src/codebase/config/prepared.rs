use super::{Config, InferredRoots, RuleApplicationConfig};
use crate::config::resolve;
use crate::config::v2::{find_config_root, NoMistakesConfig};
use std::path::{Path, PathBuf};

impl Config {
    #[doc(hidden)]
    pub fn project_roots_for_rule_with_inferred(
        &self,
        root: &Path,
        rule_id: &str,
        inferred_roots: &InferredRoots,
    ) -> Vec<PathBuf> {
        let mut inferred_roots = inferred_roots.clone();
        super::project::roots_for_rule_with_inferred(
            &self.projects,
            &self.rules,
            &self.repository_rules,
            root,
            rule_id,
            &mut inferred_roots,
        )
    }

    #[doc(hidden)]
    pub fn project_roots_for_rule_application_with_inferred(
        &self,
        root: &Path,
        application: &RuleApplicationConfig,
        inferred_roots: &InferredRoots,
    ) -> Vec<PathBuf> {
        let mut inferred_roots = inferred_roots.clone();
        let mut roots = application
            .projects
            .iter()
            .filter_map(|name| self.projects.get(name))
            .map(|project| {
                project
                    .effective_root_with_cache(root, &mut inferred_roots)
                    .unwrap_or_else(|| root.to_path_buf())
            })
            .collect::<Vec<_>>();
        if application.repository {
            roots.push(root.to_path_buf());
        }
        roots.sort();
        roots.dedup();
        roots
    }
}

pub fn config_from_loaded_v2(
    start: &Path,
    config_path: Option<&Path>,
    v2: &NoMistakesConfig,
) -> Config {
    let mut config = super::conversion::config_from_v2(v2.clone());
    let gitignore_root = match config_path {
        Some(path) => resolve(start, path)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or(start.to_path_buf()),
        None => find_config_root(start),
    };
    config.augment_from_gitignore(&gitignore_root);
    config
}
