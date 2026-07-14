mod aggregate;
mod collect;
mod collect_helpers;
mod file;
mod file_parse_error;
mod file_playwright;
mod map;
mod plan;
mod playwright_facts;
mod playwright_plan;
mod runner;
mod staged_playwright;
mod stats;

pub(crate) use aggregate::playwright_aggregate_facts;
pub use collect::collect_check_facts_with_graph_files_playwright_and_sources;
pub use collect::collect_check_facts_with_playwright;
pub(crate) use collect::collect_check_facts_with_precollected_graph_facts;

pub fn collect_check_facts(
    root: &std::path::Path,
    files: Vec<std::path::PathBuf>,
    plan: CheckFactPlan,
) -> CheckFactMap {
    collect_check_facts_with_playwright(root, files, plan, None)
}

pub fn collect_check_facts_with_graph_files_and_playwright(
    root: &std::path::Path,
    files: Vec<std::path::PathBuf>,
    graph_files: Vec<std::path::PathBuf>,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
) -> CheckFactMap {
    collect::collect_check_facts_with_graph_files_and_playwright_impl(
        root,
        files,
        graph_files,
        plan,
        playwright,
        None,
    )
}
pub(crate) use collect_helpers::graph_only_files;
pub(crate) use file::{
    collect_file_facts_from_program, collect_file_facts_with_sources, is_mdx_file,
};
pub use map::CheckFactMap;
pub(crate) use map::{CheckFileFacts, PlaywrightTestFilesByProject};
pub use plan::CheckFactPlan;
pub(crate) use playwright_facts::PlaywrightTestFacts;
pub use playwright_plan::PlaywrightFactPlan;
pub(crate) use playwright_plan::{
    PlaywrightFactSelection, PlaywrightOccurrenceKey, PlaywrightSettingsKey,
};
pub(crate) use runner::{collect_prepared_runner_facts, runner_config_facts, RunnerConfigFacts};
pub use stats::CheckFactStats;

#[cfg(test)]
pub(crate) mod tests;
