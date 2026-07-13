use super::types::Options;
use crate::config::v2::schema::NoMistakesConfig;
use std::path::{Path, PathBuf};

mod story_patterns;
pub(super) use story_patterns::{extract_storybook_story_patterns, project_relative_pattern};

pub(super) fn effective_story_patterns(
    root: &Path,
    project_root: &Path,
    config: &NoMistakesConfig,
    opts: &Options,
) -> Vec<String> {
    if !opts.stories.is_empty() {
        return opts.stories.clone();
    }
    let mut patterns = Vec::new();
    if let Some(configs) = config.tests.storybook.configs.as_ref() {
        for config_path in configs.values() {
            let config_path = resolve_storybook_config_path(root, project_root, &config_path);
            let Ok(source) = std::fs::read_to_string(&config_path) else {
                continue;
            };
            let base = config_path.parent().unwrap_or(project_root);
            for story in extract_storybook_story_patterns(&source) {
                patterns.push(project_relative_pattern(project_root, base, &story));
            }
        }
    }
    if patterns.is_empty() {
        patterns.push("**/*.stories.{ts,tsx,js,jsx}".to_string());
    }
    patterns.sort();
    patterns.dedup();
    patterns
}

fn resolve_storybook_config_path(root: &Path, project_root: &Path, config_path: &str) -> PathBuf {
    let path = Path::new(config_path);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let from_project = project_root.join(path);
    if from_project.exists() {
        from_project
    } else {
        root.join(path)
    }
}
