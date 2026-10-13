use clap::Args;
use no_mistakes::cli::Format;
use std::path::PathBuf;

#[derive(Args)]
pub(crate) struct GraphCommandArgs {
    /// Project root directory.
    #[arg(long, default_value = ".", global = true)]
    pub(crate) root: PathBuf,
    /// Path to tsconfig.json for path alias resolution.
    #[arg(long, global = true)]
    pub(crate) tsconfig: Option<PathBuf>,
    /// Filter to files matching this glob. Can be repeated.
    #[arg(long = "filter", global = true)]
    pub(crate) filters: Vec<String>,
    /// Maximum edge traversal depth for edge commands when roots are provided.
    /// Defaults to 1 when roots are provided, and unlimited otherwise.
    #[arg(long, alias = "max-depth", global = true)]
    pub(crate) depth: Option<usize>,
    /// Output format: json, yml, md, paths, human.
    #[arg(
        long,
        value_enum,
        default_value = "human",
        global = true,
        conflicts_with = "json"
    )]
    pub(crate) format: Format,
    /// Shorthand for --format json.
    #[arg(long, global = true, conflicts_with = "format")]
    pub(crate) json: bool,
    /// Legacy programmatic timing switch. CLI timing flags are root-global.
    #[arg(skip)]
    pub(crate) timings: bool,
}
