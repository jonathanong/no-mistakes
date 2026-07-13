use super::{ConfigFile, Runner};
use crate::config::v2::{ConfigView, NoMistakesConfig};
use globset::Glob;
use std::path::{Path, PathBuf};

pub(in super::super) fn config_files_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    visible_files: &[PathBuf],
) -> Vec<ConfigFile> {
    let view = ConfigView::new(config);
    let mut configured = expand_config_patterns_from_visible(
        root,
        view.vitest_configs().unwrap_or_default(),
        Runner::Vitest,
        visible_files,
    );
    configured.extend(expand_config_patterns_from_visible(
        root,
        view.jest_configs().unwrap_or_default(),
        Runner::Jest,
        visible_files,
    ));
    let configured = normalize_configs(configured);
    if !configured.is_empty() {
        return configured;
    }
    discovered_configs_from_visible(root, visible_files)
}

fn discovered_configs_from_visible(root: &Path, visible_files: &[PathBuf]) -> Vec<ConfigFile> {
    let discovered = [
        "vitest.config.ts",
        "vitest.config.mts",
        "vitest.config.cts",
        "vitest.config.js",
        "vitest.config.mjs",
        "vitest.config.cjs",
    ]
    .into_iter()
    .map(|path| ConfigFile {
        path: root.join(path),
        runner: Runner::Vitest,
    })
    .chain(jest_config_names().into_iter().map(|path| ConfigFile {
        path: root.join(path),
        runner: Runner::Jest,
    }));
    normalize_configs(
        discovered
            .filter(|config| {
                let config_path = crate::codebase::ts_resolver::normalize_path(&config.path);
                visible_files
                    .iter()
                    .any(|path| crate::codebase::ts_resolver::normalize_path(path) == config_path)
            })
            .collect(),
    )
}

fn jest_config_names() -> [&'static str; 7] {
    [
        "jest.config.ts",
        "jest.config.mts",
        "jest.config.cts",
        "jest.config.js",
        "jest.config.mjs",
        "jest.config.cjs",
        "jest.config.json",
    ]
}

fn normalize_configs(configs: Vec<ConfigFile>) -> Vec<ConfigFile> {
    configs
        .into_iter()
        .map(|config| ConfigFile {
            path: crate::codebase::ts_resolver::normalize_path(&config.path),
            runner: config.runner,
        })
        .collect()
}

fn expand_config_patterns_from_visible(
    root: &Path,
    patterns: Vec<String>,
    runner: Runner,
    visible_files: &[PathBuf],
) -> Vec<ConfigFile> {
    if patterns.is_empty() {
        return Vec::new();
    }
    let mut configs = Vec::new();
    for pattern in patterns {
        if is_glob(&pattern) {
            if let Ok(glob) = Glob::new(&pattern) {
                let matcher = glob.compile_matcher();
                for file in visible_files {
                    let rel = crate::codebase::ts_source::relative_slash_path(root, file);
                    if matcher.is_match(rel) {
                        configs.push(ConfigFile {
                            path: file.clone(),
                            runner,
                        });
                    }
                }
            }
        } else {
            let path = root.join(pattern);
            if path.exists() {
                configs.push(ConfigFile { path, runner });
            }
        }
    }
    configs
}

fn is_glob(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?') || pattern.contains('[')
}
