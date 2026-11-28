#[derive(Args, Debug, Clone)]
pub struct WhyArgs {
    /// Project root directory.
    #[arg(long, default_value = ".")]
    pub root: PathBuf,

    /// Path to config file.
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Path to tsconfig.json for alias resolution.
    #[arg(long)]
    pub tsconfig: Option<PathBuf>,

    /// The selected or skipped test file to explain.
    #[arg(required = true)]
    pub test: PathBuf,

    /// Changed file to explain the connection path from.
    #[arg(long)]
    pub changed: Option<PathBuf>,

    /// Path to a previously generated plan JSON file.
    #[arg(long)]
    pub plan: Option<PathBuf>,

    /// Output format: text, json.
    #[arg(long, value_enum, default_value = "text")]
    pub format: WhyFormat,
}

#[derive(Args, Debug, Clone)]
pub struct CommentArgs {
    /// Path to the plan JSON file.
    #[arg(required = true)]
    pub plan: PathBuf,

    /// Output file path to write the comment to (defaults to stdout).
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct GraphArgs {
    /// Path to the plan JSON file.
    #[arg(required = true)]
    pub plan: PathBuf,

    /// Output format: mermaid, json.
    #[arg(long, value_enum, default_value = "mermaid")]
    pub format: GraphFormat,

    /// Output file path (defaults to stdout).
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestFramework {
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
pub enum PlanFormat {
    Json,
    Paths,
    Commands,
    Explain,
    Markdown,
    Md,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactFormat {
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
pub struct TargetsArgs {
    /// Test framework whose project/config ownership should be resolved.
    #[arg(value_enum)]
    pub framework: TestFramework,

    /// Test files to look up.
    #[arg(required = true)]
    pub files: Vec<PathBuf>,

    /// Project root directory.
    #[arg(long, default_value = ".")]
    pub root: PathBuf,

    /// Path to config file.
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, conflicts_with = "json")]
    pub format: Option<TargetsFormat>,

    /// Shorthand for --format json.
    #[arg(long, default_value_t = false, conflicts_with = "format")]
    pub json: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetsFormat {
    Json,
    Commands,
    Paths,
    Human,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyFormat {
    Text,
    Json,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphFormat {
    Mermaid,
    Json,
}

#[derive(Args, Debug, Clone)]
pub struct AuditArgs {
    /// Versioned plan envelope JSON (including caller-recorded provenance).
    #[arg(long)]
    pub plan: PathBuf,
    /// Versioned full-suite per-test-file execution observations JSON.
    #[arg(long)]
    pub observations: PathBuf,
    /// Output format.
    #[arg(long, value_enum, default_value = "json")]
    pub format: AuditFormat,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditFormat {
    Json,
    Text,
}
