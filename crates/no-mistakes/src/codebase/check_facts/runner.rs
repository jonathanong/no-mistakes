use super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(crate) type RunnerConfigFacts = std::collections::BTreeMap<
    PathBuf,
    crate::integration_tests::runner_config::RunnerConfigFileFacts,
>;

pub(crate) fn collect_prepared_runner_facts(
    root: &Path,
    files: &[PathBuf],
    graph_only_files: &[PathBuf],
    plan: &CheckFactPlan,
    graph_plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> (
    (HashMap<PathBuf, CheckFileFacts>, RunnerConfigFacts),
    HashMap<PathBuf, CheckFileFacts>,
) {
    let (runner_files, _) = split_runner_config_files(files, plan);
    let (runner_graph_files, _) = split_runner_config_files(graph_only_files, plan);
    let collect = || {
        let mut facts = super::collect::collect_fact_map_sequential_with_sources(
            root,
            &runner_files,
            plan,
            playwright,
            &sources,
        );
        facts.extend(super::collect::collect_fact_map_sequential_with_sources(
            root,
            &runner_graph_files,
            graph_plan,
            playwright,
            &sources,
        ));
        let mut configs = runner_config_facts(&facts);
        if let Some(runner_plan) = &plan.integration_runner_configs {
            for path in runner_plan.paths() {
                if configs.contains_key(path) {
                    continue;
                }
                if let Some(config) = runner_plan.parse_path_for_facts(path) {
                    configs.insert(path.clone(), config);
                }
            }
        }
        (facts, configs)
    };
    let Some(runner_plan) = &plan.integration_runner_configs else {
        return (collect(), HashMap::new());
    };
    let fact_plan = crate::integration_tests::runner_config::RunnerConfigFactPlan {
        root: root.to_path_buf(),
        primary_files: files
            .iter()
            .map(|path| crate::codebase::ts_resolver::normalize_path(path))
            .collect(),
        graph_files: graph_only_files
            .iter()
            .map(|path| crate::codebase::ts_resolver::normalize_path(path))
            .collect(),
        primary_plan: plan.clone(),
        graph_plan: graph_plan.clone(),
        playwright: playwright.cloned(),
    };
    runner_plan.with_request_cache_and_sources(
        Some(fact_plan),
        Some(std::sync::Arc::clone(&sources)),
        collect,
    )
}

fn split_runner_config_files(
    files: &[PathBuf],
    plan: &CheckFactPlan,
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let Some(runner_configs) = &plan.integration_runner_configs else {
        return (Vec::new(), files.to_vec());
    };
    files
        .iter()
        .cloned()
        .partition(|path| runner_configs.contains(path))
}

pub(crate) fn runner_config_facts(facts: &HashMap<PathBuf, CheckFileFacts>) -> RunnerConfigFacts {
    facts
        .iter()
        .filter_map(|(path, facts)| {
            facts
                .integration_runner_config
                .clone()
                .map(|runner| (path.clone(), runner))
        })
        .collect()
}
