mod coverage_paths;
mod globs;
mod projects;
mod workflow_filters;

use super::RuleFinding;
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::Result;
use coverage_paths::{coverage_paths, CoveragePath};
use globs::selected_by_paths_filter;
use projects::{coverage_units_with_catalog, CoverageUnit};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use workflow_filters::{ci_filters_from_snapshot_with_sources, WorkflowSelector};

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

#[doc(hidden)]
pub fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(root, all_files);
    check_with_files_from_snapshot_and_catalog(root, config, all_files, &snapshot, None)
}

pub(crate) fn check_with_files_from_snapshot_and_catalog(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    catalog: Option<&super::PreparedVitestProjectCatalog>,
) -> Result<Vec<RuleFinding>> {
    let sources = snapshot.source_store_for(root);
    check_with_files_from_snapshot_catalog_and_sources(
        root, config, all_files, snapshot, catalog, &sources,
    )
}

pub(crate) fn check_with_files_from_snapshot_catalog_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    catalog: Option<&super::PreparedVitestProjectCatalog>,
    sources: &crate::codebase::ts_source::SourceStore,
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
            scan_with_catalog_and_sources(
                root,
                config,
                (&opts, &files, all_files),
                snapshot,
                catalog,
                sources,
            )
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan_with_catalog_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    inputs: (&Options, &[PathBuf], &[PathBuf]),
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    catalog: Option<&super::PreparedVitestProjectCatalog>,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let (opts, files, all_files) = inputs;
    if files.is_empty() && all_files.is_empty() {
        return Ok(Vec::new());
    }

    let (filters, mut findings) =
        ci_filters_from_snapshot_with_sources(root, config, &opts.workflows, snapshot, sources);
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

    for unit in coverage_units_with_catalog(root, config, opts, catalog)? {
        let path_files = if unit.source.uses_all_files() {
            all_files
        } else {
            files
        };
        let paths = coverage_paths(root, &unit, path_files)?;
        if paths.is_empty() {
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
        for path in paths {
            if mapped_filters.iter().any(|filter| {
                filter.workflow_allows(&path.rel)
                    && selected_by_paths_filter(&filter.compiled, filter.quantifier, &path.rel)
            }) {
                continue;
            }
            findings.push(missed_path_finding(&mapped_filters, &unit, path));
        }
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
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
            unit.project, unit.source.label(), unit.project
        ),
        import: None,
        target: Some(unit.project.clone()),
    }
}

fn missed_path_finding(
    filters: &[&workflow_filters::CiFilter],
    unit: &CoverageUnit,
    path: CoveragePath,
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
            "{}: Vitest project `{}` {}{} is not covered by CI path filters: {filter_list}",
            path.rel,
            unit.project,
            unit.source.label(),
            if path.synthetic {
                " glob witness path"
            } else {
                " path"
            }
        ),
        import: None,
        target: Some(path.rel),
    }
}

#[cfg(test)]
mod tests;
