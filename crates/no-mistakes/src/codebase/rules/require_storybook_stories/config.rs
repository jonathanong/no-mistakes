use super::types::Options;
use crate::codebase::ts_resolver::{load_tsconfig, normalize_path, TsConfig};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn resolve_tsconfig(root: &Path, tsconfig_path: Option<&Path>) -> Result<TsConfig> {
    match tsconfig_path {
        Some(path) => load_tsconfig(path),
        None => match crate::codebase::ts_resolver::find_tsconfig(root) {
            Some(path) => load_tsconfig(&path),
            None => Ok(TsConfig {
                dir: root.to_path_buf(),
                paths: vec![],
                paths_dir: root.to_path_buf(),
                base_url: None,
            }),
        },
    }
}

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
    let from_root = root.join(path);
    if from_root.exists() {
        from_root
    } else {
        project_root.join(path)
    }
}

fn extract_storybook_story_patterns(source: &str) -> Vec<String> {
    let Some(stories_start) = source.find("stories") else {
        return Vec::new();
    };
    let Some(array_start) = source[stories_start..]
        .find('[')
        .map(|idx| stories_start + idx)
    else {
        return Vec::new();
    };
    let Some(array_end) = source[array_start..].find(']').map(|idx| array_start + idx) else {
        return Vec::new();
    };
    let array = &source[array_start + 1..array_end];
    let mut out = Vec::new();
    let mut chars = array.chars().peekable();
    while let Some(quote) = chars.next() {
        if quote != '\'' && quote != '"' {
            continue;
        }
        let mut value = String::new();
        let mut escaped = false;
        for ch in chars.by_ref() {
            if escaped {
                value.push(ch);
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                if !value.is_empty() {
                    out.push(value);
                }
                break;
            } else {
                value.push(ch);
            }
        }
    }
    out
}

fn project_relative_pattern(project_root: &Path, base: &Path, pattern: &str) -> String {
    let pattern_path = Path::new(pattern);
    if pattern_path.is_absolute() {
        return pattern.to_string();
    }
    let joined = base.join(pattern_path);
    relative_slash_path(project_root, &normalize_path(&joined))
}
