mod globs;
mod projects;
mod workflow_filters;

use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::{Context, Result};
use globs::{compile_patterns, selected_by};
use projects::{coverage_units, CoverageUnit};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use workflow_filters::{ci_filters, WorkflowSelector};

pub const RULE_ID: &str = "vitest-ci-path-coverage";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) project_filters: BTreeMap<String, Vec<String>>,
    pub(crate) source_globs_by_project: BTreeMap<String, Vec<String>>,
    pub(crate) workflows: Vec<WorkflowSelector>,
    pub(crate) include_vitest_project_globs: Option<bool>,
    pub(crate) include_full_suite_triggers: Option<bool>,
    pub(crate) explicit_projects_only: bool,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, config, &opts, &files)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(
    root: &Path,
    config: &NoMistakesConfig,
    opts: &Options,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    if files.is_empty() {
        return Ok(Vec::new());
    }

    let rel_files = files
        .iter()
        .map(|path| relative_slash_path(root, path))
        .collect::<Vec<_>>();
    let (filters, mut findings) = ci_filters(root, config, &opts.workflows);
    let filters_by_name = filters.iter().fold(
        BTreeMap::<&str, Vec<&workflow_filters::CiFilter>>::new(),
        |mut acc, filter| {
            acc.entry(filter.name.as_str()).or_default().push(filter);
            acc
        },
    );
    let fallback_file = filters
        .first()
        .map(|filter| filter.workflow.as_str())
        .unwrap_or(".github/workflows");

    for unit in coverage_units(root, config, opts)? {
        let matched_files = files_matching_unit(&unit, &rel_files)?;
        if matched_files.is_empty() {
            continue;
        }
        let mapped_names = mapped_filter_names(opts, &unit.project);
        let mapped_filters = mapped_names
            .iter()
            .flat_map(|name| {
                filters_by_name
                    .get(name.as_str())
                    .into_iter()
                    .flatten()
                    .copied()
            })
            .collect::<Vec<_>>();
        if mapped_filters.is_empty() {
            findings.push(missing_mapping_finding(fallback_file, &unit));
            continue;
        }
        for rel in matched_files {
            if mapped_filters
                .iter()
                .any(|filter| selected_by(&filter.compiled, &rel))
            {
                continue;
            }
            findings.push(missed_path_finding(&mapped_filters, &unit, rel));
        }
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

fn files_matching_unit(unit: &CoverageUnit, rel_files: &[String]) -> Result<Vec<String>> {
    let compiled = compile_patterns(&unit.patterns)
        .with_context(|| format!("invalid glob in {RULE_ID} {}", unit.project))?;
    Ok(rel_files
        .iter()
        .filter(|rel| selected_by(&compiled, rel))
        .cloned()
        .collect())
}

fn mapped_filter_names(opts: &Options, project: &str) -> Vec<String> {
    opts.project_filters
        .get(project)
        .filter(|names| !names.is_empty())
        .cloned()
        .unwrap_or_else(|| vec![project.to_string()])
}

fn missing_mapping_finding(file: &str, unit: &CoverageUnit) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message: format!(
            "Vitest project `{}` {} paths are not mapped to any CI path filter; configure options.projectFilters.{}",
            unit.project, unit.source, unit.project
        ),
        import: None,
        target: Some(unit.project.clone()),
    }
}

fn missed_path_finding(
    filters: &[&workflow_filters::CiFilter],
    unit: &CoverageUnit,
    rel: String,
) -> RuleFinding {
    let filter_list = filters
        .iter()
        .map(|filter| format!("{}:{}", filter.workflow, filter.name))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: filters[0].workflow.clone(),
        line: 1,
        message: format!(
            "{rel}: Vitest project `{}` {} path is not covered by CI path filters: {filter_list}",
            unit.project, unit.source
        ),
        import: None,
        target: Some(rel),
    }
}

#[cfg(test)]
mod tests;
