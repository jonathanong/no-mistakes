#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum, Default)]
pub enum Include {
    #[default]
    Exports,
    Imports,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, Default)]
#[clap(rename_all = "kebab-case")]
pub enum SymbolsMode {
    #[default]
    List,
    SignatureImpact,
}

/// `--kind` filter values, validated by clap at parse time. Maps 1:1 onto
/// `crate::codebase::ts_symbols::ExportKind` so a typo like `--kind functoin` is rejected
/// with a helpful error instead of silently producing an empty result set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum ExportKindArg {
    Function,
    Class,
    Const,
    Let,
    Var,
    Type,
    Interface,
    Enum,
    Default,
    ReExport,
}

impl ExportKindArg {
    /// Returns `true` iff this arg matches the given `ExportKind` extracted from a file.
    fn matches(&self, k: &ExportKind) -> bool {
        matches!(
            (self, k),
            (Self::Function, ExportKind::Function)
                | (Self::Class, ExportKind::Class)
                | (Self::Const, ExportKind::Const)
                | (Self::Let, ExportKind::Let)
                | (Self::Var, ExportKind::Var)
                | (Self::Type, ExportKind::TypeAlias)
                | (Self::Interface, ExportKind::Interface)
                | (Self::Enum, ExportKind::Enum)
                | (Self::Default, ExportKind::Default)
                | (Self::ReExport, ExportKind::ReExport { .. })
        )
    }
}

/// CLI args for the `symbols` binary.
#[derive(Parser, Debug)]
pub struct SymbolsArgs {
    /// One or more TS/JS files to inspect (relative to --root or absolute).
    #[arg(required = true, value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Project root (default: current working directory).
    #[arg(long, value_name = "PATH")]
    pub root: Option<PathBuf>,

    /// Path to tsconfig.json for resolving re-export / import specifiers.
    /// If omitted, searches upward from --root.
    #[arg(long, value_name = "FILE")]
    pub tsconfig: Option<PathBuf>,

    /// Path to no-mistakes config for test classification in impact mode.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Report mode. `list` preserves the default symbols output.
    #[arg(long, value_enum, default_value_t = SymbolsMode::List)]
    pub mode: SymbolsMode,

    /// Exported symbol to analyze with `--mode signature-impact`.
    #[arg(long, value_name = "SYMBOL")]
    pub symbol: Option<String>,

    /// Only include exports of this kind. Repeatable. Validated by clap.
    #[arg(long = "kind", value_enum, value_name = "KIND")]
    pub kinds: Vec<ExportKindArg>,

    /// Which sections to emit: `exports` (default), `imports`, or `both`.
    #[arg(long, value_enum, default_value_t = Include::Exports)]
    pub include: Include,

    /// Output format: json, md, yml, paths, human.
    /// Defaults to human on TTY, json on non-TTY.
    #[arg(long, value_name = "FORMAT")]
    pub format: Option<Format>,

    /// Shorthand for `--format json`.
    #[arg(long, default_value_t = false)]
    pub json: bool,

    /// Emit phase timings to stderr.
    #[arg(long, default_value_t = false)]
    pub timings: bool,
}

/// One file's extracted symbols, ready to render.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Path relative to `--root` (or absolute if outside the root).
    pub rel_path: PathBuf,
    /// Symbols, with re-export sources resolved when possible.
    pub exports: Vec<ResolvedExport>,
    pub imports: Vec<ResolvedImport>,
}

/// An export with its re-export source resolved (when applicable) to a project-relative path.
#[derive(Debug, Clone)]
pub struct ResolvedExport {
    pub name: String,
    pub kind: ExportKind,
    pub line: u32,
    /// Resolved re-export target path, relative to `--root`. `None` for non-re-exports
    /// or when the source can't be resolved (e.g. bare npm specifier).
    pub resolved: Option<PathBuf>,
}

/// An import with the source specifier resolved (when applicable) to a project-relative path.
#[derive(Debug, Clone)]
pub struct ResolvedImport {
    pub source: String,
    pub imported: String,
    pub local: String,
    pub line: u32,
    pub is_type_only: bool,
    /// Resolved target path, relative to `--root`. `None` for bare npm specifiers.
    pub resolved: Option<PathBuf>,
}
