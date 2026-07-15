pub struct ImportResolver<'a> {
    tsconfig: &'a TsConfig,
    visible: Option<&'a HashSet<PathBuf>>,
    alias_order: Vec<usize>,
    policy: ImportResolutionPolicy<'a>,
    cache_enabled: bool,
    cache: std::sync::Arc<ResolverResultCache>,
    shared_cache: Option<&'a ImportResolutionCache>,
    session_scoped: bool,
    observer: Option<std::sync::Arc<crate::diagnostics::InvocationObserver>>,
}

#[derive(Clone, Copy)]
enum ImportResolutionPolicy<'a> {
    Standard,
    QueueCompatibility { root: &'a Path },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ResolveKey {
    importing_file: PathBuf,
    specifier: String,
}

pub(crate) type ResolverResultCache = DashMap<ResolveKey, Option<PathBuf>>;

/// Exact identity for a resolver cache within one analysis invocation.
///
/// Store the complete effective config and visible universe rather than a hash
/// fingerprint so distinct resolution semantics can never share an entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ResolverScopeKey {
    dir: PathBuf,
    paths: Vec<(String, Vec<String>)>,
    paths_dir: PathBuf,
    base_url: Option<PathBuf>,
    visible: Option<Vec<PathBuf>>,
}

impl ResolverScopeKey {
    pub(crate) fn new(tsconfig: &TsConfig, visible: Option<&HashSet<PathBuf>>) -> Self {
        let visible = visible.map(|paths| {
            let mut paths = paths.iter().cloned().collect::<Vec<_>>();
            paths.sort();
            paths
        });
        Self {
            dir: tsconfig.dir.clone(),
            paths: tsconfig.paths.clone(),
            paths_dir: tsconfig.paths_dir.clone(),
            base_url: tsconfig.base_url.clone(),
            visible,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportClassification {
    resolver_target: Option<PathBuf>,
    workspace_target: Option<PathBuf>,
    workspace_recognized: bool,
}

impl ImportClassification {
    pub(crate) fn preferred_path(&self) -> Option<&Path> {
        self.resolver_target
            .as_deref()
            .or(self.workspace_target.as_deref())
    }

    pub(crate) fn resolver_path(&self) -> Option<&Path> {
        self.resolver_target.as_deref()
    }

    pub(crate) fn workspace_path(&self) -> Option<&Path> {
        self.workspace_target.as_deref()
    }

    pub(crate) fn is_unresolved_external(&self) -> bool {
        self.resolver_target.is_none()
            && self.workspace_target.is_none()
            && !self.workspace_recognized
    }
}

/// Request-scoped memo table shared by graph consumers that classify imports.
///
/// The cache owns only resolution outcomes, so sharing it does not make lazy
/// traversal prepare any other graph consumer.
#[derive(Default)]
pub(crate) struct ImportResolutionCache {
    raw_entries: DashMap<ResolveKey, Option<PathBuf>>,
    final_entries: DashMap<ResolveKey, ImportClassification>,
    classifications: std::sync::atomic::AtomicUsize,
    requests: std::sync::atomic::AtomicUsize,
}

impl ImportResolutionCache {
    pub(crate) fn clear(&self) {
        self.raw_entries.clear();
        self.final_entries.clear();
    }
}
