use super::super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use super::helpers::{collect_test_partition, graph_plan, needs_scoped_facts, with_imports};
use super::partitions::FilePartitions;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

pub(super) fn collect_partitions(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    partitions: &FilePartitions,
    plan: &mut CheckFactPlan,
    playwright: &PlaywrightFactPlan,
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &mut HashMap<PathBuf, CheckFileFacts>,
) {
    let test_partitions = [
        (&partitions.scoped_tests, with_imports(plan.clone())),
        (&partitions.graph_tests, with_imports(graph_plan(plan))),
        (
            &partitions.playwright_only_tests,
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
    if !plan.graph.is_empty() {
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
