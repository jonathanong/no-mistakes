#[derive(Clone, clap::Parser)]
pub struct TraverseArgs {
    /// Programmatic API symbol entrypoints parallel to `files`.
    #[arg(skip)]
    pub file_symbols: Vec<Option<String>>,

    /// Programmatic API object-form entrypoints parallel to `files`.
    #[arg(skip)]
    pub file_entrypoints_are_structured: Vec<bool>,

    /// Project root directory (default: current working directory).
    #[arg(long, value_name = "PATH")]
    pub root: Option<PathBuf>,

    /// Path to tsconfig.json for path alias resolution.
    /// If omitted, searches upward from root for tsconfig.json.
    #[arg(long, value_name = "FILE")]
    pub tsconfig: Option<PathBuf>,

    /// Maximum traversal depth (default: unlimited). Alias: `--max-depth`.
    #[arg(long, alias = "max-depth", value_name = "N")]
    pub depth: Option<usize>,

    /// Only include files matching this glob pattern. Can be repeated (OR logic).
    /// Patterns ending in `/` collapse results to that folder level.
    #[arg(long = "filter", value_name = "GLOB")]
    pub filters: Vec<String>,

    /// Only include external module nodes matching this glob. Can be repeated (OR logic).
    #[arg(long = "target-module", value_name = "GLOB")]
    pub target_modules: Vec<String>,

    /// Filter to test files for a specific framework. Can be repeated.
    /// Values: vitest, playwright, cargo, dotnet, swift, python, go, rails, php.
    #[arg(long = "test", value_name = "FRAMEWORK")]
    pub tests: Vec<String>,

    /// Output format: json, md, yml, paths, human.
    /// Defaults to human on TTY, json on non-TTY.
    #[arg(long, value_name = "FORMAT", conflicts_with = "json")]
    pub format: Option<Format>,

    /// Shorthand for `--format json`.
    #[arg(long, default_value_t = false, conflicts_with = "format")]
    pub json: bool,

    /// Only follow edges of this relationship kind. Can be repeated (OR logic).
    /// Values: call, import, import-static, import-dynamic, import-type, import-require, route-import, workspace, package, test, route, queue, md, ci, workflow, http, process, asset, react, resource, dotnet, swift, terraform, python, go, rust, ruby, php, all.
    /// Default: all.
    #[arg(long = "relationship", value_enum, value_name = "KIND")]
    pub relationships: Vec<RelationshipArg>,

    /// Include exported symbol nodes in graph traversal and output.
    #[arg(long = "symbols", default_value_t = false)]
    pub include_symbols: bool,

    /// Pre-traversal include globs for the initial candidate inventory.
    /// Import-only `dependencies` only. Unrelated excluded files are not parsed.
    #[arg(long = "candidate-include", value_name = "GLOB")]
    pub candidate_include: Vec<String>,

    /// Pre-traversal exclude globs for the initial candidate inventory.
    /// Import-only `dependencies` only. Reachable imports still escape this set.
    #[arg(long = "candidate-exclude", value_name = "GLOB")]
    pub candidate_exclude: Vec<String>,

    /// Result projection. `paths` is a compact path-only JSON object.
    #[arg(long = "projection", value_enum, default_value_t = TraverseProjection::Graph)]
    pub projection: TraverseProjection,

    /// Legacy programmatic timing switch. CLI timing flags are root-global.
    #[arg(skip)]
    pub timings: bool,

    /// Files to start from. Supports `FILE#SYMBOL` for symbol-level dependents queries
    /// and `QUEUE_FILE#JOB_NAME` for queue-job dependents queries.
    /// Can be relative to --root or absolute.
    #[arg(required = true, value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

impl Default for TraverseArgs {
    fn default() -> Self {
        Self {
            file_symbols: Vec::new(),
            file_entrypoints_are_structured: Vec::new(),
            root: None,
            tsconfig: None,
            depth: None,
            filters: Vec::new(),
            target_modules: Vec::new(),
            tests: Vec::new(),
            format: None,
            json: false,
            relationships: Vec::new(),
            include_symbols: false,
            candidate_include: Vec::new(),
            candidate_exclude: Vec::new(),
            projection: TraverseProjection::Graph,
            timings: false,
            files: Vec::new(),
        }
    }
}

impl TraverseArgs {
    pub(crate) fn has_candidate_bounds(&self) -> bool {
        !self.candidate_include.is_empty() || !self.candidate_exclude.is_empty()
    }
}
