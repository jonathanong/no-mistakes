use anyhow::Result;
use no_mistakes::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use no_mistakes::codebase::test_discovery::TestExecutionTarget;
use no_mistakes::codebase::ts_resolver::TsConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::ExitCode;

pub(crate) mod args;
pub(crate) mod changed_files;
pub(crate) mod comment;
pub(crate) mod configured_plan;
pub(crate) mod configured_plan_candidates;
pub(crate) mod diff_parser;
pub(crate) mod graph;
pub(crate) mod impact;
pub(crate) mod lockfile_changes;
pub(crate) mod plan;
pub(crate) mod plan_output;
pub(crate) mod prepared_plan;
pub(crate) mod targets;
pub(crate) mod why;

pub use args::TestsArgs;
pub(crate) use args::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TestPlan {
    pub selected_tests: Vec<SelectedTest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<TestPlanGroupResult>,
    pub warnings: Vec<Warning>,
    pub fallback_triggered: bool,
    pub fallback_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SelectedTest {
    pub test_file: String,
    pub confidence: Confidence,
    pub reasons: Vec<ImpactReason>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<TestExecutionTarget>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TestPlanGroupResult {
    pub r#type: String,
    pub selected: Vec<String>,
    pub remaining: usize,
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Low = 0,
    Medium = 1,
    High = 2,
}

impl Confidence {
    pub fn display_emoji(self) -> &'static str {
        match self {
            Confidence::Low => "🔴 Low",
            Confidence::Medium => "🟡 Medium",
            Confidence::High => "🟢 High",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ImpactReason {
    pub changed_file: String,
    pub path: Vec<String>,
    pub via: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    pub r#type: String,
    pub message: String,
    pub file: String,
}

pub fn run(args: TestsArgs) -> Result<ExitCode> {
    match args.command {
        TestsCommand::Plan(sub_args) => plan::run(*sub_args),
        TestsCommand::Targets(sub_args) => targets::run(sub_args),
        TestsCommand::Impact(sub_args) => impact::run(sub_args),
        TestsCommand::Why(sub_args) => why::run(sub_args),
        TestsCommand::Comment(sub_args) => comment::run(sub_args),
        TestsCommand::Graph(sub_args) => graph::run(sub_args),
    }
}

const _: fn(TestsArgs) -> Result<ExitCode> = run;

/// Build the canonical graph shape shared by both test-impact entry points.
pub(crate) fn build_test_impact_graph(
    root: &Path,
    tsconfig: &TsConfig,
    include_symbols: bool,
) -> Result<DepGraph> {
    DepGraph::build_with_plan(
        root,
        tsconfig,
        GraphBuildPlan::test_impact().with_symbols(include_symbols),
    )
}
