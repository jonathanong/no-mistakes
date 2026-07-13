use super::coverage::build_coverage;
use super::fetch::{
    collect_fetches_for_routes_from_snapshot, collect_fetches_for_routes_from_snapshot_with_facts,
    expand_fetch_edges,
};
use super::pipeline_setup::PlaywrightSetup;
use super::types::{Analysis, CoverageInputs, EdgeReport, TestFileAnalysis, UniqueSelectorPolicy};
use crate::playwright::config::Settings;
use anyhow::Result;
use std::path::Path;

pub(crate) fn finish_analysis(
    root: &Path,
    settings: &Settings,
    unique_selector_policy: UniqueSelectorPolicy,
    setup: PlaywrightSetup,
    test_analysis: TestFileAnalysis,
    facts: Option<&dyn crate::codebase::dependencies::graph::TsFactLookup>,
    snapshot: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Analysis> {
    let route_root = root.join(&settings.frontend_root);
    let mut edges = test_analysis.edges;
    let fetch_idx = if setup.routes.is_empty() {
        Default::default()
    } else {
        crate::perf_trace::trace("playwright.fetches_for_routes", || {
            match facts.filter(|facts| facts.playwright_source_files().is_some()) {
                Some(facts) => collect_fetches_for_routes_from_snapshot_with_facts(
                    setup.routes.as_slice(),
                    &route_root,
                    root,
                    snapshot,
                    facts,
                ),
                None => collect_fetches_for_routes_from_snapshot(
                    setup.routes.as_slice(),
                    &route_root,
                    root,
                    snapshot,
                ),
            }
        })?
    };
    edges.extend(expand_fetch_edges(&edges, &fetch_idx));
    edges.sort();
    edges.dedup();
    let edge_report = EdgeReport { edges };
    let coverage = crate::perf_trace::trace("playwright.build_coverage", || {
        build_coverage(CoverageInputs {
            root,
            routes: setup.routes.as_slice(),
            app_selectors: &setup.app_selectors,
            app_selector_occurrences: setup.app_selector_occurrences.as_slice(),
            edges: &edge_report.edges,
            helper_references: &test_analysis.helper_references,
            settings,
            unique_selector_policy,
            fetch_index: &fetch_idx,
        })
    });
    Ok(Analysis {
        coverage,
        edges: edge_report,
    })
}
