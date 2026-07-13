use super::{collect_fact_map, CheckFactMap, CheckFactPlan, CheckFactStats, PlaywrightFactPlan};
use dashmap::DashMap;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(super) fn collect(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    graph_files_complete: bool,
    mut plan: CheckFactPlan,
    playwright: PlaywrightFactPlan,
) -> CheckFactMap {
    let partitions = FilePartitions::new(&files, &graph_files, &playwright);
    let mut ts = HashMap::new();
    collect_test_partition(
        root,
        &partitions.scoped_tests,
        with_imports(plan.clone()),
        &playwright,
        &mut ts,
    );
    collect_test_partition(
        root,
        &partitions.graph_tests,
        with_imports(graph_plan(&plan)),
        &playwright,
        &mut ts,
    );
    collect_test_partition(
        root,
        &partitions.playwright_only_tests,
        with_imports(CheckFactPlan::default()),
        &playwright,
        &mut ts,
    );

    let playwright_facts = ts
        .iter()
        .filter_map(|(path, facts)| facts.playwright.as_ref().map(|facts| (path.clone(), facts)))
        .collect::<BTreeMap<_, _>>();
    if playwright.demands_text_imports(&playwright_facts) {
        plan.graph.imports = true;
    }
    if needs_scoped_facts(&plan) {
        ts.extend(collect_fact_map(
            root,
            &partitions.remaining_scoped,
            &plan,
            Some(&playwright),
        ));
    }
    if !plan.graph.is_empty() {
        ts.extend(collect_fact_map(
            root,
            &partitions.remaining_graph,
            &graph_plan(&plan),
            Some(&playwright),
        ));
    }
    finish_map(
        files,
        graph_files,
        graph_files_complete,
        plan,
        partitions.files_discovered,
        ts,
    )
}

fn collect_test_partition(
    root: &Path,
    files: &[PathBuf],
    plan: CheckFactPlan,
    playwright: &PlaywrightFactPlan,
    facts: &mut HashMap<PathBuf, super::CheckFileFacts>,
) {
    facts.extend(collect_fact_map(root, files, &plan, Some(playwright)));
}

fn with_imports(mut plan: CheckFactPlan) -> CheckFactPlan {
    plan.graph.imports = true;
    plan
}

fn graph_plan(plan: &CheckFactPlan) -> CheckFactPlan {
    CheckFactPlan {
        graph: plan.graph,
        graph_context: plan.graph_context.clone(),
        ..CheckFactPlan::default()
    }
}

fn needs_scoped_facts(plan: &CheckFactPlan) -> bool {
    plan.imports
        || plan.symbols
        || plan.react
        || plan.queue
        || plan.integration
        || plan.dynamic_imports
        || plan.nextjs_caching
        || plan.storybook
        || plan.source
        || plan.raw_source
        || !plan.graph.is_empty()
}

fn finish_map(
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    graph_files_complete: bool,
    plan: CheckFactPlan,
    files_discovered: usize,
    ts: HashMap<PathBuf, super::CheckFileFacts>,
) -> CheckFactMap {
    let files_parsed = ts.values().filter(|facts| facts.parsed).count();
    let parse_errors = ts
        .values()
        .filter(|facts| facts.parse_error.is_some())
        .count();
    CheckFactMap {
        files,
        graph_files,
        graph_files_complete,
        ts,
        graph_plan: plan.graph,
        stats: CheckFactStats {
            files_discovered,
            files_parsed,
            parse_errors,
        },
        app_selector_occurrences_cache: DashMap::new(),
        playwright_routes_cache: OnceLock::new(),
        app_text_targets_cache: OnceLock::new(),
        route_reachable_files_cache: OnceLock::new(),
    }
}

struct FilePartitions {
    scoped_tests: Vec<PathBuf>,
    graph_tests: Vec<PathBuf>,
    playwright_only_tests: Vec<PathBuf>,
    remaining_scoped: Vec<PathBuf>,
    remaining_graph: Vec<PathBuf>,
    files_discovered: usize,
}

impl FilePartitions {
    fn new(files: &[PathBuf], graph_files: &[PathBuf], playwright: &PlaywrightFactPlan) -> Self {
        let scoped: BTreeSet<_> = files.iter().cloned().collect();
        let graph: BTreeSet<_> = graph_files.iter().cloned().collect();
        let tests: BTreeSet<_> = playwright.paths().cloned().collect();
        let graph_only: BTreeSet<_> = graph.difference(&scoped).cloned().collect();
        let known: BTreeSet<_> = scoped.union(&graph).cloned().collect();
        let discovered: BTreeSet<_> = known.union(&tests).cloned().collect();
        Self {
            scoped_tests: tests.intersection(&scoped).cloned().collect(),
            graph_tests: tests.intersection(&graph_only).cloned().collect(),
            playwright_only_tests: tests.difference(&known).cloned().collect(),
            remaining_scoped: scoped.difference(&tests).cloned().collect(),
            remaining_graph: graph_only.difference(&tests).cloned().collect(),
            files_discovered: discovered.len(),
        }
    }
}

#[cfg(test)]
mod tests;
