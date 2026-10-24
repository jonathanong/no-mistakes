use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "tsconfig-alias-folder-mapping";

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct AliasMapping {
    pub(crate) prefix: String,
    pub(crate) root: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) tsconfig: PathBuf,
    pub(crate) base_dir: String,
    pub(crate) mappings: Vec<AliasMapping>,
    /// When true, verify literal targets and wildcard parent directories exist.
    pub(crate) check_exists: bool,
}

mod mapping;
mod targets;
use mapping::check_alias;
use targets::{build_path, target_exists, target_matches};

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.try_rule_options()?;
            if opts.tsconfig.as_os_str().is_empty() || opts.mappings.is_empty() {
                return Ok(vec![]);
            }
            let findings = check_tsconfig(root, &opts)?;
            if findings.is_empty() {
                return Ok(vec![]);
            }
            let filter = super::path_filter::RulePathFilter::new(root, config, rule)?;
            let mut filtered = Vec::new();
            for finding in findings {
                if filter.is_match(&root.join(&opts.tsconfig)) {
                    filtered.push(finding);
                }
            }
            Ok(filtered)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

/// Check using a pre-discovered file list — tsconfig is read directly regardless.
pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    _files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    check(root, config)
}

fn check_tsconfig(root: &Path, opts: &Options) -> Result<Vec<RuleFinding>> {
    let tsconfig_path = root.join(&opts.tsconfig);
    // A missing path is an intentional skip. A file that exists but cannot be
    // loaded must not pass silently: its alias mappings were not checked.
    if !tsconfig_path.is_file() {
        return Ok(Vec::new());
    }
    let (ts_config, locations) =
        match crate::codebase::ts_resolver::load_tsconfig_with_path_locations(&tsconfig_path) {
            Ok(config) => config,
            Err(err) => {
                let file = relative_slash_path(root, &tsconfig_path);
                let message = format!(
                "{file}: tsconfig could not be loaded and alias mappings were not checked: {err}"
            );
                return Ok(vec![make_finding(&file, 1, &message)]);
            }
        };
    if ts_config.paths.is_empty() {
        return Ok(Vec::new());
    }
    // Extends paths are normalized by the loader; callers may supply roots
    // containing `..`. Compare both lexical forms before rendering a relative path.
    let source_path = locations
        .as_ref()
        .map_or(tsconfig_path.as_path(), |locations| &locations.file);
    let tsconfig_str = relative_slash_path(
        &crate::codebase::ts_resolver::normalize_path(root),
        &crate::codebase::ts_resolver::normalize_path(source_path),
    );
    let all_paths: BTreeMap<String, Vec<String>> = ts_config.paths.into_iter().collect();
    let mut findings = Vec::new();
    for (alias, targets) in &all_paths {
        findings.extend(check_alias(
            root,
            &tsconfig_str,
            locations
                .as_ref()
                .and_then(|locations| locations.lines.get(alias))
                .copied()
                .unwrap_or(1),
            alias,
            targets,
            opts,
            &all_paths,
        ));
    }
    Ok(findings)
}

fn make_finding(file: &str, line: usize, message: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: message.to_string(),
        import: None,
        target: None,
    }
}

#[cfg(test)]
mod tests;
