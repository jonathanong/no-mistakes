//! Opt-in check for Vite `optimizeDeps.include` entries shaped `parent > child`.

use super::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod config_object;
mod entry_imports;
mod include_config;
mod manifest;
mod scan;
mod specifier;

pub const RULE_ID: &str = "vite-optimize-deps-nested-imports";

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct Options {
    config_files: Option<Vec<String>>,
    packages: Option<Vec<String>>,
}

super::borrowed_source_rule_entrypoints!();

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    _all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
        let anchor =
            super::config_diagnostic::ConfigAnchor::resolve(root, config, Some(sources), rule);
        let mut batch = match required(&opts) {
            Ok((_configs, packages)) if packages.is_empty() => Vec::new(),
            Ok((configs, packages)) => scan::evaluate(root, sources, &configs, &packages)?,
            Err((key, message)) => vec![anchor.emit(
                RULE_ID,
                anchor.option_line(key),
                format!("{RULE_ID}: {message}"),
                None,
                None,
            )],
        };
        if let Some(prefix) = &rule.message {
            for finding in &mut batch {
                finding.message = format!("{prefix} {}", finding.message);
            }
        }
        findings.extend(batch);
    }
    if !defer_suppression {
        super::suppress_rule_findings_with_sources(root, &mut findings, sources);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn required(
    opts: &Options,
) -> std::result::Result<(Vec<String>, Vec<String>), (&'static str, String)> {
    match (&opts.config_files, &opts.packages) {
        (Some(configs), Some(packages)) => Ok((configs.clone(), packages.clone())),
        (None, None) => Err((
            "configFiles",
            "configFiles and packages are required and have no default; list the Vite config files and the package directories in scope".to_string(),
        )),
        (None, Some(_)) => Err((
            "configFiles",
            "configFiles is required and has no default; list the Vite config files to read".to_string(),
        )),
        (Some(_), None) => Err((
            "packages",
            "packages is required and has no default; list the package directories in scope".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests;
