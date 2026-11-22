use super::fragments::Catalog;
use super::import_resolution::FragmentResolver;
use super::scan_file::one;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::check_outcome::CheckFailure;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::postgres::embedded::TrustedSqlTag;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use rayon::prelude::*;
use std::path::Path;

pub(super) struct ScanReport {
    pub findings: Vec<RuleFinding>,
    pub gaps: Vec<CheckFailure>,
}

pub(super) struct FileScan {
    pub findings: Vec<RuleFinding>,
    pub coverage_gaps: Vec<CheckFailure>,
    pub gaps: Vec<CheckFailure>,
}

pub(super) fn scan(
    root: &Path,
    tags: &[TrustedSqlTag],
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
    resolution: Option<&dyn FragmentResolver>,
) -> ScanReport {
    if !super::has_trusted_tags(tags) {
        return ScanReport {
            findings: Vec::new(),
            gaps: Vec::new(),
        };
    }
    let mut catalog = Catalog::build(facts, tags, sources, resolution);
    let rows: Vec<_> = facts
        .graph_file_universe()
        .par_iter()
        .filter_map(|file| one(root, tags, graph, facts, sources, &catalog, file))
        .collect();
    let mut findings = Vec::new();
    let mut gaps = Vec::new();
    let mut coverage_gaps = Vec::new();
    for row in rows {
        findings.extend(row.findings);
        gaps.extend(row.gaps);
        coverage_gaps.extend(row.coverage_gaps);
    }
    findings.sort();
    findings.dedup();
    let proved: crate::fx::FxHashSet<&str> =
        findings.iter().map(|item| item.file.as_str()).collect();
    let unresolved = catalog.take_unresolved();
    if !coverage_gaps.is_empty() {
        let relevant =
            super::gap_reachability::relevant_files(root, &catalog, graph, facts, &proved);
        gaps.extend(coverage_gaps.into_iter().filter(|gap| {
            gap.file
                .as_ref()
                .is_some_and(|file| relevant.contains(file))
        }));
    }
    gaps.extend(unresolved.into_iter().filter_map(|(file, specifier)| {
        let relative = relative_slash_path(root, &file);
        if !proved.contains(relative.as_str())
            && !super::relevance::reaches_fragment(&catalog, graph, facts, &file)
        {
            return None;
        }
        Some(CheckFailure {
            consumer: super::RULE_ID.to_string(),
            file: Some(file),
            property: None,
            message: format!(
                "`{relative}` could not resolve import `{specifier}`, so an escape of an exported SQL fragment may be missing. Resolve the import and rerun the check."
            ),
        })
    }));
    gaps.sort();
    gaps.dedup();
    ScanReport { findings, gaps }
}

pub(super) fn missing_facts(root: &Path, file: &Path) -> Option<FileScan> {
    is_indexable(file).then(|| {
        let relative = relative_slash_path(root, file);
        FileScan {
            findings: Vec::new(),
            gaps: Vec::new(),
            coverage_gaps: vec![CheckFailure {
                consumer: super::RULE_ID.to_string(),
                file: Some(file.to_path_buf()),
                property: None,
                message: format!(
                    "`{relative}` is missing prepared TypeScript facts, so a write, rebind, or escape of an exported SQL fragment may be missing. Rerun the check with this file available for analysis."
                ),
            }],
        }
    })
}
