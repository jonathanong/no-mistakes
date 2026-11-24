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

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TestFramework {
    Dotnet,
    Playwright,
    Vitest,
    Swift,
    Python,
    Go,
    Cargo,
    Rails,
    Php,
    Java,
    Kotlin,
    Elixir,
    Dart,
    Jest,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlanFormat {
    Json,
    Paths,
    Commands,
    Explain,
    Markdown,
    Md,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImpactFormat {
    Json,
    Paths,
    Commands,
    Markdown,
    Md,
}

impl From<ImpactFormat> for PlanFormat {
    fn from(format: ImpactFormat) -> Self {
        match format {
            ImpactFormat::Json => Self::Json,
            ImpactFormat::Paths => Self::Paths,
            ImpactFormat::Commands => Self::Commands,
            ImpactFormat::Markdown => Self::Markdown,
            ImpactFormat::Md => Self::Md,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub(crate) struct TargetsArgs {
    /// Test framework whose project/config ownership should be resolved.
    #[arg(value_enum)]
    pub(crate) framework: TestFramework,

    /// Test files to look up.
    #[arg(required = true)]
    pub(crate) files: Vec<PathBuf>,

    /// Project root directory.
    #[arg(long, default_value = ".")]
    pub(crate) root: PathBuf,

    /// Path to config file.
    #[arg(long)]
    pub(crate) config: Option<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, conflicts_with = "json")]
    pub(crate) format: Option<TargetsFormat>,

    /// Shorthand for --format json.
    #[arg(long, default_value_t = false, conflicts_with = "format")]
    pub(crate) json: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetsFormat {
    Json,
    Commands,
    Paths,
    Human,
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

#[derive(Args, Debug, Clone)]
pub(crate) struct AuditArgs {
    /// Versioned plan envelope JSON (including caller-recorded provenance).
    #[arg(long)]
    pub(crate) plan: PathBuf,
    /// Versioned full-suite per-test-file execution observations JSON.
    #[arg(long)]
    pub(crate) observations: PathBuf,
    /// Output format.
    #[arg(long, value_enum, default_value = "json")]
    pub(crate) format: AuditFormat,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuditFormat {
    Json,
    Text,
}
