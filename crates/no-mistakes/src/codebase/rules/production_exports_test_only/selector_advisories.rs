use crate::codebase::dependencies::graph::{DepGraph, NodeId};
use crate::codebase::ts_source::relative_slash_path;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod grouping;

type Row = (PathBuf, usize, String);

#[derive(Default)]
pub(super) struct AdvisoryGroups {
    pub(super) baseline: Vec<Row>,
    compressed: Vec<Row>,
}

impl AdvisoryGroups {
    pub(super) fn findings(
        self,
        render: impl Fn(&Path, usize, &str) -> super::RuleFinding,
    ) -> (Vec<super::RuleFinding>, Vec<super::RuleFinding>) {
        let convert = |rows: Vec<Row>| {
            rows.into_iter()
                .map(|(file, line, reason)| render(&file, line, &reason))
                .collect()
        };
        (convert(self.baseline), convert(self.compressed))
    }
}

pub(super) fn choose(
    root: &Path,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    mut baseline: super::AnalysisDiagnostics,
    mut compressed: super::AnalysisDiagnostics,
) -> super::AnalysisDiagnostics {
    for result in [&mut baseline, &mut compressed] {
        crate::codebase::rules::sort_findings(&mut result.findings);
        crate::codebase::rules::sort_findings(&mut result.advisories);
    }
    let sources = session.and_then(|session| session.existing_sources_for(root));
    let visible_baseline = visible_counts(root, sources.as_deref(), &baseline);
    let visible_compressed = visible_counts(root, sources.as_deref(), &compressed);
    if visible_compressed.0 <= visible_baseline.0
        && visible_compressed.1 <= visible_baseline.1
        && compressed.findings.len() <= baseline.findings.len()
        && compressed.advisories.len() <= baseline.advisories.len()
        && compressed.findings.len() + compressed.advisories.len()
            < baseline.findings.len() + baseline.advisories.len()
    {
        compressed
    } else {
        baseline
    }
}

fn visible_counts(
    root: &Path,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
    diagnostics: &super::AnalysisDiagnostics,
) -> (usize, usize) {
    let count = |findings: &[super::RuleFinding]| {
        let mut visible = findings.to_vec();
        if let Some(sources) = sources {
            crate::codebase::rules::suppress_rule_findings_with_sources(
                root,
                &mut visible,
                sources,
            );
        }
        visible.len()
    };
    (count(&diagnostics.findings), count(&diagnostics.advisories))
}

type SiteKey = (PathBuf, usize, Option<String>);
type Reasons = BTreeSet<&'static str>;
type Sites = BTreeMap<SiteKey, Reasons>;
type ExportEvidence = BTreeMap<(PathBuf, String), (String, Sites)>;

#[derive(Default)]
pub(super) struct SelectorAdvisories {
    exports: ExportEvidence,
}

impl SelectorAdvisories {
    pub(super) fn append(&mut self, other: Self) {
        self.exports.extend(other.exports);
    }

    pub(super) fn insert(
        &mut self,
        graph: &DepGraph,
        file: &Path,
        key: &str,
        name: &str,
        unknown: &BTreeSet<(PathBuf, NodeId)>,
    ) {
        if unknown.is_empty() {
            return;
        }
        let (_, evidence) = self
            .exports
            .entry((file.to_path_buf(), key.to_string()))
            .or_insert_with(|| (name.to_string(), Sites::new()));
        for (importer, target) in unknown {
            let sites = graph.unknown_selector_sites(importer, target);
            if sites.is_empty() {
                evidence.entry((importer.clone(), 1, None)).or_default().insert(
                    "missing prepared selector provenance: the graph lacks the import line, module specifier, and selector reason; rebuild analysis with symbol import selector facts and their source evidence, or replace the import/re-export with explicit names",
                );
            }
            for site in sites {
                evidence
                    .entry((importer.clone(), site.line, Some(site.specifier.clone())))
                    .or_default()
                    .insert(site.reason);
            }
        }
    }

    pub(super) fn finish(self, root: &Path) -> AdvisoryGroups {
        grouping::finish(root, self.exports)
    }
}

/// Type-only importers of an export whose value-or-type kind is unknown.
pub(super) fn undetermined(
    root: &Path,
    graph: &DepGraph,
    typed: &BTreeSet<(PathBuf, NodeId)>,
    name: &str,
) -> (PathBuf, usize, String) {
    let mut evidence = BTreeSet::new();
    for (importer, target) in typed {
        let path = relative_slash_path(root, importer);
        let line = graph
            .unknown_selector_sites(importer, target)
            .first()
            .map_or(1, |site| site.line);
        evidence.insert((importer.clone(), line, format!("{path}:{line}")));
    }
    let (file, line, _) = evidence.first().cloned().expect("typed importers exist");
    let sites = evidence
        .into_iter()
        .map(|(_, _, site)| site)
        .collect::<Vec<_>>()
        .join(", ");
    (
        file,
        line,
        format!(
            "type-only references to `{name}` ({sites}) cannot be classified because its export kind \
             (value or type) is undetermined"
        ),
    )
}
