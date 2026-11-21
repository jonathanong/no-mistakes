use super::fragments::Catalog;
use super::scan_file::one;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::check_outcome::CheckFailure;
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::postgres::embedded::TrustedSqlTag;
use crate::codebase::rules::RuleFinding;
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
    if !tags
        .iter()
        .any(|tag| !tag.module.is_empty() && !tag.name.is_empty())
    {
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
    gaps.sort();
    gaps.dedup();
    ScanReport { findings, gaps }
}
