/// A stable, machine-readable problem found while constructing or selecting
/// the request-local TypeScript configuration catalog.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct TsConfigDiagnostic {
    pub(crate) kind: TsConfigDiagnosticKind,
    pub(crate) config: Option<PathBuf>,
    pub(crate) file: Option<PathBuf>,
    pub(crate) detail: String,
    pub(crate) candidates: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum TsConfigDiagnosticKind {
    AmbiguousOwnership,
    InvalidConfig,
    InvalidExtends,
    InvalidReference,
}

impl TsConfigDiagnostic {
    fn config(kind: TsConfigDiagnosticKind, config: &Path, detail: impl Into<String>) -> Self {
        Self {
            kind,
            config: Some(config.to_path_buf()),
            file: None,
            detail: detail.into(),
            candidates: Vec::new(),
        }
    }
}

/// The configuration selected for an importing source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TsConfigProvenance {
    pub(crate) importer: PathBuf,
    pub(crate) config: Option<PathBuf>,
    pub(crate) forced: bool,
}

/// Request-scoped, deterministic TypeScript configuration ownership.
#[doc(hidden)]
pub struct TsConfigCatalog {
    configs: Vec<CatalogConfig>,
    broken_dirs: Vec<PathBuf>,
    empty: TsConfig,
    forced: bool,
    build_diagnostics: BTreeSet<TsConfigDiagnostic>,
    diagnostics: Mutex<BTreeSet<TsConfigDiagnostic>>,
}

struct CatalogConfig {
    path: PathBuf,
    config: TsConfig,
    matcher: ConfigMatcher,
    module_resolution: Option<String>,
    identity: Vec<PathBuf>,
}

#[derive(Clone)]
struct ConfigMatcher {
    dir: PathBuf,
    real_dir: PathBuf,
    files: Option<BTreeSet<PathBuf>>,
    includes: Option<Vec<GlobRule>>,
    excludes: Vec<GlobRule>,
    out_dir: Option<PathBuf>,
    allow_js: bool,
}

#[derive(Clone)]
struct GlobRule {
    base: PathBuf,
    matcher: globset::GlobMatcher,
    allow_parent: bool,
    absolute: bool,
}
