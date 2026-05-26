use clap::{Args, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub(crate) struct TestsArgs {
    #[command(subcommand)]
    pub(crate) command: TestsCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum TestsCommand {
    /// Plan test targets based on changed files and dependency graph analysis.
    Plan(PlanArgs),
    /// Find impacted tests from file#symbol entrypoints.
    Impact(ImpactArgs),
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

    /// Path to a unified diff file.
    #[arg(long, conflicts_with_all = ["diff_stdin", "diff_command"])]
    pub(crate) diff: Option<PathBuf>,

    /// Read unified diff from stdin.
    #[arg(long, default_value_t = false, conflicts_with_all = ["diff", "diff_command"])]
    pub(crate) diff_stdin: bool,

    /// Run a command and parse its stdout as a unified diff.
    #[arg(long = "diff-command", conflicts_with_all = ["diff", "diff_stdin"])]
    pub(crate) diff_command: Option<String>,

    /// file#export entrypoints to trace (union of all). Can be repeated.
    #[arg(long = "entrypoint")]
    pub(crate) entrypoints: Vec<String>,

    /// Inline diff content (programmatic API only).
    #[arg(skip)]
    pub(crate) diff_content: Option<String>,

    /// Test plan environment name from config.
    #[arg(long, default_value = "pre-push")]
    pub(crate) environment: String,

    /// Override the configured plan limit percentage.
    #[arg(long = "limit-percent")]
    pub(crate) limit_percent: Option<f64>,

    /// Override the configured plan limit file count.
    #[arg(long = "limit-files")]
    pub(crate) limit_files: Option<usize>,

    /// Override whether global config changes trigger full-suite fallback.
    #[arg(long = "global-config-fallback")]
    pub(crate) global_config_fallback: Option<bool>,

    /// Output format.
    #[arg(long, value_enum, conflicts_with = "json")]
    pub(crate) format: Option<PlanFormat>,

    /// Shorthand for --format json.
    #[arg(long, default_value_t = false, conflicts_with = "format")]
    pub(crate) json: bool,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct ImpactArgs {
    /// file#export entrypoints (positional, repeatable).
    #[arg(required = true)]
    pub(crate) entrypoints: Vec<String>,

    /// Project root directory.
    #[arg(long, default_value = ".")]
    pub(crate) root: PathBuf,

    /// Path to config file.
    #[arg(long)]
    pub(crate) config: Option<PathBuf>,

    /// Path to tsconfig.json for alias resolution.
    #[arg(long)]
    pub(crate) tsconfig: Option<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, conflicts_with = "json")]
    pub(crate) format: Option<PlanFormat>,

    /// Shorthand for --format json.
    #[arg(long, default_value_t = false, conflicts_with = "format")]
    pub(crate) json: bool,
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

    /// Output file path (defaults to stdout).
    #[arg(long)]
    pub(crate) out: Option<PathBuf>,
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

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhyFormat {
    Text,
    Json,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GraphFormat {
    Mermaid,
    Json,
}
