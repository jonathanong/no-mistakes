use super::super::PlaywrightFactPlan;
use std::collections::BTreeSet;
use std::path::PathBuf;

pub(super) struct FilePartitions {
    pub(super) scoped_tests: Vec<PathBuf>,
    pub(super) graph_tests: Vec<PathBuf>,
    pub(super) playwright_only_tests: Vec<PathBuf>,
    pub(super) scoped_sources: Vec<PathBuf>,
    pub(super) graph_sources: Vec<PathBuf>,
    pub(super) playwright_only_sources: Vec<PathBuf>,
    pub(super) remaining_scoped: Vec<PathBuf>,
    pub(super) remaining_graph: Vec<PathBuf>,
    pub(super) files_discovered: usize,
}

impl FilePartitions {
    pub(super) fn new(
        files: &[PathBuf],
        graph_files: &[PathBuf],
        playwright: &PlaywrightFactPlan,
    ) -> Self {
        let scoped: BTreeSet<_> = files.iter().cloned().collect();
        let graph: BTreeSet<_> = graph_files.iter().cloned().collect();
        let tests: BTreeSet<_> = playwright.paths().cloned().collect();
        let sources: BTreeSet<_> = playwright.source_files().iter().cloned().collect();
        let configs: BTreeSet<_> = playwright.config_files().iter().cloned().collect();
        let source_only: BTreeSet<_> = sources.difference(&tests).cloned().collect();
        let graph_only: BTreeSet<_> = graph.difference(&scoped).cloned().collect();
        let known: BTreeSet<_> = scoped.union(&graph).cloned().collect();
        let test_or_source: BTreeSet<_> = tests.union(&sources).cloned().collect();
        let discovered: BTreeSet<_> = known.union(&test_or_source).cloned().collect();
        let planned_sources: BTreeSet<_> = tests.union(&source_only).cloned().collect();
        let planned: BTreeSet<_> = planned_sources.union(&configs).cloned().collect();
        Self {
            scoped_tests: tests.intersection(&scoped).cloned().collect(),
            graph_tests: tests.intersection(&graph_only).cloned().collect(),
            playwright_only_tests: tests.difference(&known).cloned().collect(),
            scoped_sources: source_only.intersection(&scoped).cloned().collect(),
            graph_sources: source_only.intersection(&graph_only).cloned().collect(),
            playwright_only_sources: source_only.difference(&known).cloned().collect(),
            remaining_scoped: scoped.difference(&planned).cloned().collect(),
            remaining_graph: graph_only.difference(&planned).cloned().collect(),
            files_discovered: discovered.len(),
        }
    }
}
