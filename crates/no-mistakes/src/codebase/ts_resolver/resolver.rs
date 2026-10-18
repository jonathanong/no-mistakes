pub struct ImportResolver<'a> {
    tsconfig: ResolverTsConfig<'a>,
    visible: Option<ResolverVisible<'a>>,
    alias_order: Vec<usize>,
    policy: ImportResolutionPolicy<'a>,
    cache_enabled: bool,
    cache: std::sync::Arc<ResolverResultCache>,
    shared_cache: Option<&'a ImportResolutionCache>,
    session_scoped: bool,
    session_exact_visibility: bool,
    observer: Option<std::sync::Arc<crate::diagnostics::InvocationObserver>>,
    interner: Option<Arc<crate::codebase::analysis_session::PathInterner>>,
}

/// The ordinary resolver borrows its config. Long-lived, request-scoped
/// consumers can instead retain an `Arc` without tying the resolver to a
/// self-reference into their owner.
enum ResolverTsConfig<'a> {
    Borrowed(&'a TsConfig),
    Owned(std::sync::Arc<TsConfig>),
}

impl ResolverTsConfig<'_> {
    fn get(&self) -> &TsConfig {
        match self {
            Self::Borrowed(config) => config,
            Self::Owned(config) => config,
        }
    }
}

enum ResolverVisible<'a> {
    Borrowed(&'a dyn VisiblePathLookup),
    Owned(std::sync::Arc<dyn VisiblePathLookup>),
}

impl Clone for ResolverVisible<'_> {
    fn clone(&self) -> Self {
        match self {
            Self::Borrowed(visible) => Self::Borrowed(*visible),
            Self::Owned(visible) => Self::Owned(std::sync::Arc::clone(visible)),
        }
    }
}

impl ResolverVisible<'_> {
    fn contains_visible(&self, path: &Path) -> bool {
        self.lookup().contains_visible(path)
    }

    pub(crate) fn lookup(&self) -> &dyn VisiblePathLookup {
        match self {
            Self::Borrowed(visible) => *visible,
            Self::Owned(visible) => visible.as_ref(),
        }
    }
}

#[derive(Clone, Copy)]
enum ImportResolutionPolicy<'a> {
    Standard,
    QueueCompatibility { root: &'a Path },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ResolveKey {
    importing_file: Arc<Path>,
    specifier: Arc<str>,
}

pub(crate) type ResolverResultCache =
    DashMap<ResolveKey, Option<PathBuf>, rustc_hash::FxBuildHasher>;

/// Exact identity for a resolver cache within one analysis invocation.
///
/// Keep complete config semantics and an exact visible identity. Session-owned
/// identities retain their allocations; standalone scopes keep complete paths.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ResolverScopeKey {
    dir: PathBuf,
    paths: Vec<(String, Vec<String>)>,
    paths_dir: PathBuf,
    base_url: Option<PathBuf>,
    visible: Option<ResolverVisibleKey>,
}

/// Session cache identity for either a legacy resolver or one catalog-selected
/// config. Catalog identity stays part of the key even when effective compiler
/// options happen to be equal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ResolverCacheScopeKey {
    resolver: ResolverScopeKey,
    module_resolution: Option<String>,
    identity: Vec<PathBuf>,
    queue_root: Option<PathBuf>,
}

impl ResolverCacheScopeKey {
    pub(crate) fn with_queue_root(mut self, root: Option<&Path>) -> Self {
        self.queue_root = root.map(normalize_path);
        self
    }

    pub(crate) fn for_lookup(
        tsconfig: &TsConfig,
        visible: Option<&dyn VisiblePathLookup>,
        module_resolution: Option<&str>,
        identity: &[PathBuf],
    ) -> Self {
        let visible = visible.map(|visible| match visible.scope_identity() {
            Some(identity) => ResolverVisibleKey::Session(identity),
            None => ResolverVisibleKey::Exact(visible.visible_cache_key()),
        });
        Self {
            resolver: ResolverScopeKey {
                dir: tsconfig.dir.clone(),
                paths: tsconfig.paths.clone(),
                paths_dir: tsconfig.paths_dir.clone(),
                base_url: tsconfig.base_url.clone(),
                visible,
            },
            module_resolution: module_resolution.map(str::to_string),
            identity: identity.to_vec(),
            queue_root: None,
        }
    }
}
