use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod lists;
mod scan;

pub const RULE_ID: &str = "tsconfig-file-coverage";
pub(super) const DEFAULT_AUXILIARY_BASENAME: &str = "tsconfig.dependency-cruiser.json";

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ReasonedPath {
    pub(crate) path: String,
    pub(crate) reason: String,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct AuxiliaryConfig {
    pub(crate) path: String,
    pub(crate) reason: String,
    pub(crate) required_basename: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) allow: Vec<ReasonedPath>,
    pub(crate) auxiliary_configs: Vec<AuxiliaryConfig>,
    pub(crate) required_basename: Option<String>,
}

pub(crate) struct CompiledAuxiliary {
    pub(super) path: String,
    pub(super) reason: String,
    pub(super) required_basename: String,
}

pub(crate) struct CompiledOptions {
    pub(super) allow: Vec<ReasonedPath>,
    pub(super) auxiliary: Vec<CompiledAuxiliary>,
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
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let compiled = compile_options(&opts);
        let target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots))
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan::scan(root, &compiled, &files, sources));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> CompiledOptions {
    let default_basename = opts
        .required_basename
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(DEFAULT_AUXILIARY_BASENAME);
    CompiledOptions {
        allow: opts
            .allow
            .iter()
            .map(|entry| ReasonedPath {
                path: normalize_rel(&entry.path),
                reason: entry.reason.clone(),
            })
            .collect(),
        auxiliary: opts
            .auxiliary_configs
            .iter()
            .map(|entry| CompiledAuxiliary {
                path: normalize_rel(&entry.path),
                reason: entry.reason.clone(),
                required_basename: entry
                    .required_basename
                    .as_deref()
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .unwrap_or(default_basename)
                    .to_string(),
            })
            .collect(),
    }
}

pub(super) fn normalize_rel(path: &str) -> String {
    path.replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect::<Vec<_>>()
        .join("/")
}

pub(super) fn finding(file: &str, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message,
        import: None,
        target: Some(file.to_string()),
    }
}

#[cfg(test)]
mod tests;
