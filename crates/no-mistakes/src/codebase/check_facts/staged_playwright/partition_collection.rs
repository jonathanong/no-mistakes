use super::super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use super::helpers::{collect_test_partition, graph_plan, needs_scoped_facts, with_imports};
use super::partitions::FilePartitions;
use crate::codebase::ts_source::FileIdMap;
use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn collect_partitions(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    partitions: &FilePartitions,
    plan: &mut CheckFactPlan,
    playwright: &PlaywrightFactPlan,
    sources: &crate::codebase::ts_source::SourceStore,
    supplemental: super::super::SupplementalFactInput<'_, &mut FileIdMap<CheckFileFacts>>,
) {
    let super::super::SupplementalFactInput {
        facts,
        call_sites: supplemental_call_sites,
    } = supplemental;
    let call_tests = if supplemental_call_sites.is_empty() {
        Vec::new()
    } else {
        let supplemental =
            crate::codebase::check_facts::PathMembership::new(supplemental_call_sites);
        partitions
            .playwright_only_tests
            .iter()
            .filter(|path| supplemental.contains(path))
            .cloned()
            .collect::<Vec<_>>()
    };
    let ordinary_tests = (!call_tests.is_empty()).then(|| {
        let calls = crate::codebase::check_facts::PathMembership::new(&call_tests);
        partitions
            .playwright_only_tests
            .iter()
            .filter(|path| !calls.contains(path))
            .cloned()
            .collect::<Vec<_>>()
    });
    let test_partitions = [
        (&partitions.scoped_tests, with_imports(plan.clone())),
        (&partitions.graph_tests, with_imports(graph_plan(plan))),
        (
            ordinary_tests
                .as_ref()
                .unwrap_or(&partitions.playwright_only_tests),
            with_imports(CheckFactPlan::default()),
        ),
    ];
    for (files, partition_plan) in test_partitions {
        collect_test_partition(
            session,
            root,
            files,
            partition_plan,
            playwright,
            sources,
            facts,
        );
    }
    if !call_tests.is_empty() {
        let mut call_plan = with_imports(CheckFactPlan::default());
        call_plan.graph.call_sites = true;
        collect_test_partition(
            session,
            root,
            &call_tests,
            call_plan,
            playwright,
            sources,
            facts,
        );
    }
    let playwright_facts = facts
        .iter()
        .filter_map(|(path, facts)| facts.playwright.as_ref().map(|facts| (path.clone(), facts)))
        .collect::<BTreeMap<_, _>>();
    if playwright.demands_text_imports(&playwright_facts) {
        plan.graph
            .include(crate::codebase::ts_source::facts::TsFactPlan::imports());
    }
    let source_partitions = [
        (&partitions.scoped_sources, plan.clone()),
        (&partitions.graph_sources, graph_plan(plan)),
        (&partitions.playwright_only_sources, graph_plan(plan)),
    ];
    for (files, partition_plan) in source_partitions {
        collect_test_partition(
            session,
            root,
            files,
            partition_plan,
            playwright,
            sources,
            facts,
        );
    }
    if needs_scoped_facts(plan) {
        collect_test_partition(
            session,
            root,
            &partitions.remaining_scoped,
            plan.clone(),
            playwright,
            sources,
            facts,
        );
    }
    if !plan.graph.is_empty() || !plan.call_site_paths.is_empty() {
        collect_test_partition(
            session,
            root,
            &partitions.remaining_graph,
            graph_plan(plan),
            playwright,
            sources,
            facts,
        );
    }
}
