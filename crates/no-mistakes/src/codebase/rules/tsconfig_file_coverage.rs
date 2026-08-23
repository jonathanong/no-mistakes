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
    pub(super) invalid: Vec<(String, String)>,
}

/// Run coverage against `all_files`. The filesystem dispatcher passes the
/// tracked inventory so untracked scratch files are omitted. Direct callers
/// should pass `snapshot.tracked_paths_from(files)` when a discovery snapshot
/// is available; otherwise this uses the supplied list unchanged.
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
        let opts: Options = rule.rule_options()?;
        let compiled = compile_options(&opts);
        let target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let in_scope: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots))
            .cloned()
            .collect();
        let candidates = super::path_filter::filter_rule_files(root, config, rule, &in_scope)?;
        findings.extend(scan::scan(root, &compiled, all_files, &candidates, sources));
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
    let mut invalid = Vec::new();
    let allow = opts
        .allow
        .iter()
        .filter_map(|entry| compiled_reasoned("allow", entry, &mut invalid))
        .collect();
    let auxiliary = opts
        .auxiliary_configs
        .iter()
        .filter_map(|entry| {
            let path = compiled_path("auxiliaryConfigs", &entry.path, &mut invalid)?;
            Some(CompiledAuxiliary {
                path,
                reason: entry.reason.clone(),
                required_basename: entry
                    .required_basename
                    .as_deref()
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .unwrap_or(default_basename)
                    .to_string(),
            })
        })
        .collect();
    CompiledOptions {
        allow,
        auxiliary,
        invalid,
    }
}

fn compiled_reasoned(
    kind: &str,
    entry: &ReasonedPath,
    invalid: &mut Vec<(String, String)>,
) -> Option<ReasonedPath> {
    Some(ReasonedPath {
        path: compiled_path(kind, &entry.path, invalid)?,
        reason: entry.reason.clone(),
    })
}

fn compiled_path(kind: &str, path: &str, invalid: &mut Vec<(String, String)>) -> Option<String> {
    match normalize_rel(path) {
        Some(normalized) => Some(normalized),
        None => {
            invalid.push((kind.to_string(), path.to_string()));
            None
        }
    }
}

pub(super) fn normalize_rel(path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    if crate::codebase::ts_source::is_portably_absolute_path(Path::new(path)) {
        return None;
    }
    let mut parts = Vec::new();
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => return None,
            part => parts.push(part),
        }
    }
    Some(parts.join("/"))
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
