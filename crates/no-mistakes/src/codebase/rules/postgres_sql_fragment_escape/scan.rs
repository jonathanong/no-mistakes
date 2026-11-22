use super::fragments::Catalog;
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

pub(super) fn scan(
    root: &Path,
    tags: &[TrustedSqlTag],
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> ScanReport {
    if !super::has_trusted_tags(tags) {
        return ScanReport {
            findings: Vec::new(),
            gaps: Vec::new(),
        };
    }
    let catalog = Catalog::build(facts, tags, sources);
    let mut rows: Vec<ScanReport> = facts
        .graph_file_universe()
        .par_iter()
        .filter_map(|file| one(root, tags, graph, facts, sources, &catalog, file))
        .collect();
    rows.sort_by(|left, right| {
        left.findings
            .cmp(&right.findings)
            .then(left.gaps.cmp(&right.gaps))
    });
    let mut findings = Vec::new();
    let mut gaps = Vec::new();
    for row in rows {
        findings.extend(row.findings);
        gaps.extend(row.gaps);
    }
    findings.sort();
    findings.dedup();
    gaps.extend(catalog.into_unproven().map(|(file, specifier)| {
        let relative = relative_slash_path(root, &file);
        CheckFailure {
            consumer: super::RULE_ID.to_string(),
            file: Some(file),
            property: None,
            message: format!(
                "`{relative}` could not resolve bare specifier `{specifier}` without the canonical import graph, so a write, rebind, or escape of an exported SQL fragment may be missing. Fix the graph prerequisite and rerun the check."
            ),
        }
    }));
    gaps.sort();
    gaps.dedup();
    ScanReport { findings, gaps }
}

pub(super) fn missing_facts(root: &Path, file: &Path) -> Option<ScanReport> {
    is_indexable(file).then(|| {
        let relative = relative_slash_path(root, file);
        ScanReport {
            findings: Vec::new(),
            gaps: vec![CheckFailure {
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
