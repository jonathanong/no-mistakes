use super::RULE_ID;
use anyhow::{anyhow, bail, Result};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use regex::Regex;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(in crate::codebase::rules) struct Options {
    pub(super) single: Vec<ApiTarget>,
    pub(super) bulk: Vec<ApiTarget>,
    pub(super) single_export_pattern: Option<String>,
    pub(super) bulk_export_pattern: Option<String>,
    pub(super) module_globs: Vec<String>,
}

impl Options {
    pub(super) fn has_single_targets(&self) -> bool {
        !self.single.is_empty() || self.single_export_pattern.is_some()
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct ApiTarget {
    pub(super) module: String,
    pub(super) export: String,
}

#[derive(Debug)]
pub(in crate::codebase::rules) struct Targets {
    pub(super) single: Vec<(String, String)>,
    pub(super) bulk: Vec<(String, String)>,
    pub(super) single_export_pattern: Option<Regex>,
    pub(super) bulk_export_pattern: Option<Regex>,
    pub(super) module_globs: Option<GlobSet>,
}

pub(in crate::codebase::rules) fn validate(options: &Options) -> Result<Targets> {
    let single = collect("single", &options.single)?;
    let bulk = collect("bulk", &options.bulk)?;
    let single_pairs: BTreeSet<_> = single.iter().cloned().collect();
    if let Some((module, export)) = bulk.iter().find(|pair| single_pairs.contains(pair)) {
        bail!(
            "{RULE_ID}: `{module}` export `{export}` is listed in both options.single and options.bulk"
        );
    }
    Ok(Targets {
        single,
        bulk,
        single_export_pattern: compile_pattern(
            "singleExportPattern",
            options.single_export_pattern.as_deref(),
        )?,
        bulk_export_pattern: compile_pattern(
            "bulkExportPattern",
            options.bulk_export_pattern.as_deref(),
        )?,
        module_globs: compile_module_globs(&options.module_globs)?,
    })
}

fn compile_pattern(field: &str, pattern: Option<&str>) -> Result<Option<Regex>> {
    pattern
        .map(|pattern| {
            if pattern.trim().is_empty() {
                bail!("{RULE_ID}: options.{field} needs a non-empty regex");
            }
            Regex::new(pattern).map_err(|error| {
                anyhow!("{RULE_ID}: options.{field} is not a valid regex: {error}")
            })
        })
        .transpose()
}

fn compile_module_globs(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for (index, pattern) in patterns.iter().enumerate() {
        if pattern.trim().is_empty() {
            bail!("{RULE_ID}: options.moduleGlobs[{index}] needs a non-empty glob");
        }
        let glob = GlobBuilder::new(pattern)
            .literal_separator(true)
            .backslash_escape(true)
            .build()
            .map_err(|error| anyhow!("{RULE_ID}: options.moduleGlobs[{index}]: {error}"))?;
        builder.add(glob);
    }
    builder
        .build()
        .map(Some)
        .map_err(|error| anyhow!("{RULE_ID}: options.moduleGlobs: {error}"))
}

fn collect(label: &str, targets: &[ApiTarget]) -> Result<Vec<(String, String)>> {
    let mut seen = BTreeSet::new();
    let mut owned = Vec::new();
    for (index, target) in targets.iter().enumerate() {
        let module = target.module.trim();
        let export = target.export.trim();
        if module.is_empty() || export.is_empty() {
            bail!(
                "{RULE_ID}: options.{label}[{index}] needs a non-empty module specifier and export name"
            );
        }
        let pair = (module.to_string(), export.to_string());
        if !seen.insert(pair.clone()) {
            bail!("{RULE_ID}: options.{label} lists `{module}` export `{export}` more than once");
        }
        owned.push(pair);
    }
    Ok(owned)
}
