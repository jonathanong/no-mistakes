use super::types::{GlobMatcher, Options};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod story_patterns;
pub(super) use story_patterns::{extract_storybook_story_patterns, project_relative_pattern};

pub(super) fn authorize_configured_sources(
    root: &Path,
    config: &NoMistakesConfig,
    project_roots: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) {
    let Some(configs) = config.tests.storybook.configs.as_ref() else {
        return;
    };
    for config_path in configs.values() {
        for project_root in project_roots {
            let path = resolve_storybook_config_path(root, project_root, &config_path);
            if path.is_file() {
                // Explicit config paths may be ignored or outside the visible
                // inventory. SourceStore's bounded supplemental cache records
                // this exact request-authorized path without a second store.
                let _ = sources.read_path(&path);
            }
        }
    }
}

pub(crate) fn configured_story_files(
    root: &Path,
    files: &[PathBuf],
    config: &NoMistakesConfig,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<BTreeSet<PathBuf>> {
    let mut matched = BTreeSet::new();
    for rule in config.rule_applications(super::RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
        for name in &rule.projects {
            let Some(project) = config.projects.get(name) else {
                continue;
            };
            let project_root = project
                .root
                .as_deref()
                .map(|path| root.join(path))
                .unwrap_or_else(|| root.to_path_buf());
            let project_root = normalize_path(&project_root);
            let patterns = effective_story_patterns(root, &project_root, config, &opts, sources);
            let stories = GlobMatcher::new(&patterns)?;
            for path in files {
                if path.starts_with(&project_root)
                    && stories.is_match(&relative_slash_path(&project_root, path))
                {
                    matched.insert(normalize_path(path));
                }
            }
        }
    }
    Ok(matched)
}

pub(super) fn effective_story_patterns(
    root: &Path,
    project_root: &Path,
    config: &NoMistakesConfig,
    opts: &Options,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<String> {
    if !opts.stories.is_empty() {
        return opts.stories.clone();
    }
    let mut patterns = Vec::new();
    if let Some(configs) = config.tests.storybook.configs.as_ref() {
        for config_path in configs.values() {
            let config_path = resolve_storybook_config_path(root, project_root, &config_path);
            let Some(source) = crate::codebase::rules::read_source(sources, &config_path) else {
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
