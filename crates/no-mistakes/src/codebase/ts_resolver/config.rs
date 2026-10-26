#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct TsConfig {
    /// Directory containing the entry tsconfig (used as base for relative-path resolution).
    pub dir: PathBuf,
    /// `compilerOptions.paths` entries: glob pattern → list of replacement templates.
    pub paths: Vec<(String, Vec<String>)>,
    /// Directory of the tsconfig that *defines* `paths`. May differ from `dir` when the
    /// entry tsconfig inherits `paths` via `extends`. Used to anchor alias substitution.
    pub paths_dir: PathBuf,
    /// Absolute directory used for TypeScript `baseUrl` bare-specifier resolution.
    pub base_url: Option<PathBuf>,
}

/// Load and parse a `tsconfig.json` at `path`, following `extends` chains.
///
/// `compilerOptions.paths` is inherited from the nearest ancestor that defines it;
/// a child config that defines its own `paths` takes full precedence (no merge).
/// The directory of whichever config physically contains the winning `paths` block
/// is stored in `TsConfig::paths_dir` so that alias substitution is anchored there.
pub fn load_tsconfig(path: &Path) -> Result<TsConfig> {
    load_tsconfig_with_sources(path, None)
}

pub(crate) fn load_tsconfig_from_source_store(
    path: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<TsConfig> {
    load_tsconfig_with_sources(path, Some(sources))
}

fn load_tsconfig_with_sources(
    path: &Path,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> Result<TsConfig> {
    Ok(load_tsconfig_inner(path, &mut std::collections::HashSet::new(), sources, false)?.inner)
}

/// Internal result that carries whether `paths` was *explicitly defined* (even if empty)
/// somewhere in the extends chain. This lets the extends array logic correctly apply
/// TS override semantics: a later entry that defines `paths: {}` must win over an
/// earlier entry that defines `paths: { ... }`, even though the resolved vec is empty.
struct TsConfigFound {
    inner: TsConfig,
    /// `true` when `compilerOptions.paths` was present in the JSON of this config
    /// or of some config it inherited from. `false` means "not defined anywhere".
    paths_found: bool,
    locations: Option<TsConfigPathLocations>,
}

include!("config_load.rs");

/// Diagnostic metadata is opt-in so ordinary resolver loads retain no locations.
pub(crate) struct TsConfigPathLocations {
    pub(crate) file: PathBuf,
    pub(crate) lines: std::collections::BTreeMap<String, usize>,
}

pub(crate) fn load_tsconfig_with_path_locations(
    path: &Path,
) -> Result<(TsConfig, Option<TsConfigPathLocations>)> {
    let found = load_tsconfig_inner(path, &mut HashSet::new(), None, true)?;
    Ok((found.inner, found.locations))
}
