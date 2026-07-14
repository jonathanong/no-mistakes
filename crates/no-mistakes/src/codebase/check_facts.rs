mod aggregate;
mod collect;
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
pub use collect::{
    collect_check_facts, collect_check_facts_with_graph_files_and_playwright,
    collect_check_facts_with_playwright,
};
pub(crate) use collect::{
    collect_check_facts_with_precollected_graph_facts, collect_fact_map, graph_only_files,
};
pub(crate) use file::{collect_file_facts, collect_file_facts_from_program, is_mdx_file};
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
mod tests;
