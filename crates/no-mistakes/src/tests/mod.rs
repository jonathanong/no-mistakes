use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::ExitCode;

pub(crate) mod changed_files;
pub(crate) mod comment;
pub(crate) mod configured_plan;
pub(crate) mod configured_plan_candidates;
pub(crate) mod graph;
pub(crate) mod plan;
pub(crate) mod why;

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

#[derive(Args, Debug)]
pub(crate) struct TestsArgs {
    #[command(subcommand)]
    pub(crate) command: TestsCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum TestsCommand {
    /// Plan test targets based on changed files and dependency graph analysis.
    Plan(PlanArgs),
    /// Explain the dependency path from a changed file to a test file.
    Why(WhyArgs),
    /// Generate a PR comment summarizing the test plan.
    Comment(CommentArgs),
    /// Generate a visual dependency/impact relationship graph.
    Graph(GraphArgs),
}

#[derive(Args, Debug, Clone)]
pub(crate) struct PlanArgs {
    /// Optional test framework for config-driven planning.
    #[arg(value_enum)]
    pub(crate) framework: Option<TestFramework>,

    /// Project root directory.
    #[arg(long, default_value = ".")]
    pub(crate) root: PathBuf,

    /// Path to config file.
    #[arg(long)]
    pub(crate) config: Option<PathBuf>,

    /// Path to tsconfig.json for alias resolution.
    #[arg(long)]
    pub(crate) tsconfig: Option<PathBuf>,

    /// Git base commit/branch to diff against.
    #[arg(long)]
    pub(crate) base: Option<String>,

    /// Git head commit/branch to diff against (defaults to HEAD).
    #[arg(long, requires = "base")]
    pub(crate) head: Option<String>,

    /// Specific changed file path. Can be repeated.
    #[arg(long = "changed-file")]
    pub(crate) changed_file: Vec<PathBuf>,

    /// Path to a file containing a list of changed files (one per line).
    #[arg(long = "changed-files")]
    pub(crate) changed_files: Option<PathBuf>,

    /// Test plan environment name from config.
    #[arg(long, default_value = "pre-push")]
    pub(crate) environment: String,

    /// Override the configured plan limit percentage.
    #[arg(long = "limit-percent")]
    pub(crate) limit_percent: Option<f64>,

    /// Override the configured plan limit file count.
    #[arg(long = "limit-files")]
    pub(crate) limit_files: Option<usize>,

    /// Output format.
    #[arg(long, value_enum, conflicts_with = "json")]
    pub(crate) format: Option<PlanFormat>,

    /// Shorthand for --format json.
    #[arg(long, default_value_t = false, conflicts_with = "format")]
    pub(crate) json: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TestFramework {
    Playwright,
    Vitest,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlanFormat {
    Json,
    Paths,
    Markdown,
    Md,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct WhyArgs {
    /// Project root directory.
    #[arg(long, default_value = ".")]
    pub(crate) root: PathBuf,

    /// Path to config file.
    #[arg(long)]
    pub(crate) config: Option<PathBuf>,

    /// Path to tsconfig.json for alias resolution.
    #[arg(long)]
    pub(crate) tsconfig: Option<PathBuf>,

    /// The selected or skipped test file to explain.
    #[arg(required = true)]
    pub(crate) test: PathBuf,

    /// Changed file to explain the connection path from.
    #[arg(long)]
    pub(crate) changed: Option<PathBuf>,

    /// Path to a previously generated plan JSON file.
    #[arg(long)]
    pub(crate) plan: Option<PathBuf>,

    /// Output format: text, json.
    #[arg(long, value_enum, default_value = "text")]
    pub(crate) format: WhyFormat,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhyFormat {
    Text,
    Json,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct CommentArgs {
    /// Path to the plan JSON file.
    #[arg(required = true)]
    pub(crate) plan: PathBuf,

    /// Output file path to write the comment to (defaults to stdout).
    #[arg(long)]
    pub(crate) out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct GraphArgs {
    /// Path to the plan JSON file.
    #[arg(required = true)]
    pub(crate) plan: PathBuf,

    /// Output format: mermaid, json.
    #[arg(long, value_enum, default_value = "mermaid")]
    pub(crate) format: GraphFormat,

    /// Output file path to write the graph representation to (defaults to stdout).
    #[arg(long)]
    pub(crate) out: Option<PathBuf>,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GraphFormat {
    Mermaid,
    Json,
}

pub(crate) fn run(args: TestsArgs) -> Result<ExitCode> {
    match args.command {
        TestsCommand::Plan(sub_args) => plan::run(sub_args),
        TestsCommand::Why(sub_args) => why::run(sub_args),
        TestsCommand::Comment(sub_args) => comment::run(sub_args),
        TestsCommand::Graph(sub_args) => graph::run(sub_args),
    }
}
