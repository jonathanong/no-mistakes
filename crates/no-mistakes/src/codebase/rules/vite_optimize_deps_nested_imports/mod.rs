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
            Ok(scope) if scope.packages.is_empty() => Vec::new(),
            Ok(scope) => scan::evaluate(root, sources, &scope.configs, &scope.packages)?,
            Err(missing) => vec![anchor.emit(
                RULE_ID,
                anchor.option_line(missing.key),
                format!("{RULE_ID}: {}", missing.message),
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

struct RequiredScope {
    configs: Vec<String>,
    packages: Vec<String>,
}

struct MissingOption {
    key: &'static str,
    message: String,
}

fn required(opts: &Options) -> std::result::Result<RequiredScope, MissingOption> {
    match (&opts.config_files, &opts.packages) {
        (Some(configs), Some(packages)) => Ok(RequiredScope {
            configs: configs.clone(),
            packages: packages.clone(),
        }),
        (None, None) => Err(MissingOption {
            key: "configFiles",
            message: "configFiles and packages are required and have no default; list the Vite config files and the package directories in scope".to_string(),
        }),
        (None, Some(_)) => Err(MissingOption {
            key: "configFiles",
            message: "configFiles is required and has no default; list the Vite config files to read".to_string(),
        }),
        (Some(_), None) => Err(MissingOption {
            key: "packages",
            message: "packages is required and has no default; list the package directories in scope".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests;
