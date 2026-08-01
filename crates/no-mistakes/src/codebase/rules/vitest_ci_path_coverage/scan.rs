use super::super::RuleFinding;
use super::{
    coverage_paths::coverage_paths,
    findings::missed_path,
    globs::selected_by_paths_filter,
    projects::{coverage_units_with_catalog, CoverageUnit},
    workflow_filters::{
        self, ci_filters_from_parsed_with_sources, ci_filters_from_snapshot_with_sources,
    },
    Options, RULE_ID,
};
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) struct ScanInputs<'a> {
    pub(super) root: &'a Path,
    pub(super) config: &'a NoMistakesConfig,
    pub(super) opts: &'a Options,
    pub(super) files: &'a [PathBuf],
    pub(super) all_files: &'a [PathBuf],
    pub(super) snapshot: &'a crate::codebase::ts_source::VisiblePathSnapshot,
    pub(super) catalog: Option<&'a super::super::PreparedVitestProjectCatalog>,
    pub(super) sources: &'a crate::codebase::ts_source::SourceStore,
    pub(super) workflows: Option<&'a crate::codebase::ci_workflows::ParsedWorkflowSet>,
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
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn check_with_files_from_snapshot_and_catalog(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    catalog: Option<&super::super::PreparedVitestProjectCatalog>,
) -> Result<Vec<RuleFinding>> {
    let sources = snapshot.source_store_for(root);
    check_with_files_from_snapshot_catalog_and_sources(
        root, config, all_files, snapshot, catalog, &sources,
    )
}
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn check_with_files_from_snapshot_catalog_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    catalog: Option<&super::super::PreparedVitestProjectCatalog>,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    check_with_files_from_snapshot_catalog_sources_and_workflows(
        root, config, all_files, snapshot, catalog, sources, None,
    )
}
pub(crate) fn check_with_files_from_snapshot_catalog_sources_and_workflows(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    catalog: Option<&super::super::PreparedVitestProjectCatalog>,
    sources: &crate::codebase::ts_source::SourceStore,
    workflows: Option<&crate::codebase::ci_workflows::ParsedWorkflowSet>,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let opts: Options = rule.rule_options();
            let roots = super::super::target_roots(root, config, rule);
            let skip = super::super::skip_dir_set(config);
            let files = all_files
                .iter()
                .filter(|p| super::super::file_allowed_by_roots_and_skip(root, &skip, p, &roots))
                .cloned()
                .collect::<Vec<_>>();
            let files = super::super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(ScanInputs {
                root,
                config,
                opts: &opts,
                files: &files,
                all_files,
                snapshot,
                catalog,
                sources,
                workflows,
            })
        })
        .collect();
    let mut findings = all?.into_iter().flatten().collect();
    super::super::sort_findings(&mut findings);
    Ok(findings)
}
pub(super) fn scan(inputs: ScanInputs<'_>) -> Result<Vec<RuleFinding>> {
    let ScanInputs {
        root,
        config,
        opts,
        files,
        all_files,
        snapshot,
        catalog,
        sources,
        workflows,
    } = inputs;
    if files.is_empty() && all_files.is_empty() {
        return Ok(Vec::new());
    }
    let (filters, mut findings) = workflows
        .map(|workflows| {
            ci_filters_from_parsed_with_sources(root, &opts.workflows, workflows, sources)
        })
        .unwrap_or_else(|| {
            ci_filters_from_snapshot_with_sources(root, config, &opts.workflows, snapshot, sources)
        });
    let by_name = filters.iter().fold(
        BTreeMap::<&str, Vec<&workflow_filters::CiFilter>>::new(),
        |mut map, filter| {
            map.entry(&filter.name).or_default().push(filter);
            map
        },
    );
    let fallback = filters
        .first()
        .map(|filter| filter.workflow.as_str())
        .unwrap_or(".github/workflows");
    for unit in coverage_units_with_catalog(root, config, opts, catalog)? {
        let paths = coverage_paths(
            root,
            &unit,
            if unit.source.uses_all_files() {
                all_files
            } else {
                files
            },
        )?;
        let filters = mapped_filters(opts, &unit.project, &by_name);
        if filters.is_empty() {
            if !paths.is_empty() {
                findings.push(missing_mapping_finding(fallback, &unit));
            }
            continue;
        }
        for path in paths {
            if !filters.iter().any(|filter| {
                filter.workflow_allows(&path.rel)
                    && selected_by_paths_filter(&filter.compiled, filter.quantifier, &path.rel)
            }) {
                findings.push(missed_path(&filters, &unit, path));
            }
        }
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}
fn mapped_filters<'a>(
    opts: &Options,
    project: &str,
    by_name: &'a BTreeMap<&str, Vec<&'a workflow_filters::CiFilter>>,
) -> Vec<&'a workflow_filters::CiFilter> {
    mapped_filter_names(opts, project)
        .iter()
        .flat_map(|name| by_name.get(name.as_str()).into_iter().flatten().copied())
        .collect()
}

pub(super) fn mapped_filter_names(opts: &Options, project: &str) -> Vec<String> {
    opts.project_filters
        .get(project)
        .filter(|names| !names.is_empty())
        .cloned()
        .unwrap_or_else(|| vec![project.to_string()])
}
pub(super) fn missing_mapping_finding(file: &str, unit: &CoverageUnit) -> RuleFinding {
    RuleFinding { rule: RULE_ID.to_string(), file: file.to_string(), line: 1, message: format!("Vitest project `{}` {} paths are not mapped to any CI path filter; configure options.projectFilters.{}", unit.project, unit.source.label(), unit.project), import: None, target: Some(unit.project.clone()) }
}
