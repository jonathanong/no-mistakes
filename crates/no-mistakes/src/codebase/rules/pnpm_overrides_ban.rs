use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

mod keys;
use keys::{json_key_line, json_nested_key_line, yaml_has_key, yaml_top_level_key_line};

pub const RULE_ID: &str = "pnpm-overrides-ban";

const FIX: &str = "Fix dependency metadata upstream or upgrade the parent dependency instead.";

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let files = discover_files(root, &config.filesystem.skip_directories);
    check_with_files(root, config, &files)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|path| {
                    super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
                })
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            Ok(scan(root, &files, sources))
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(
    root: &Path,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    files
        .par_iter()
        .flat_map(
            |path| match path.file_name().and_then(|name| name.to_str()) {
                Some("pnpm-workspace.yaml") => check_workspace(root, path, sources),
                Some("package.json") => check_package_json(root, path, sources),
                _ => Vec::new(),
            },
        )
        .collect()
}

fn check_workspace(
    root: &Path,
    path: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let Ok(source) = sources.read_path(path) else {
        return Vec::new();
    };
    match serde_yaml::from_str::<serde_yaml::Value>(&source) {
        Ok(value) => {
            if yaml_has_key(&value, "overrides") {
                vec![finding(
                    root,
                    path,
                    yaml_top_level_key_line(&source, "overrides"),
                    format!("top-level \"overrides\" is banned. {FIX}"),
                )]
            } else {
                Vec::new()
            }
        }
        Err(error) => {
            let detail = error
                .to_string()
                .lines()
                .next()
                .unwrap_or("invalid YAML")
                .to_string();
            vec![finding(
                root,
                path,
                1,
                format!("failed to parse YAML: {detail}"),
            )]
        }
    }
}

fn check_package_json(
    root: &Path,
    path: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let Ok(source) = sources.read_path(path) else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&source) else {
        return Vec::new();
    };
    let Some(object) = json.as_object() else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    if object.contains_key("overrides") {
        findings.push(finding(
            root,
            path,
            json_key_line(&source, "overrides"),
            format!("top-level \"overrides\" is banned. {FIX}"),
        ));
    }
    if object
        .get("pnpm")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|pnpm| pnpm.contains_key("overrides"))
    {
        findings.push(finding(
            root,
            path,
            json_nested_key_line(&source, "pnpm", "overrides"),
            format!("\"pnpm.overrides\" is banned. {FIX}"),
        ));
    }
    findings
}

fn finding(root: &Path, path: &Path, line: usize, message: impl Into<String>) -> RuleFinding {
    let file = relative_slash_path(root, path);
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.clone(),
        line,
        message: format!("{file}: {}", message.into()),
        import: None,
        target: None,
    }
}

#[cfg(test)]
mod tests;
