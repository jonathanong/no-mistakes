/// Exact frozen projection. The registry retains its identity until the request
/// ends, so an address can never be recycled while a resolver key still uses it.
#[derive(Clone)]
pub struct PreparedVisible {
    paths: Arc<crate::fx::PathSet>,
    identity: VisibleScopeIdentity,
    normalized: Arc<std::sync::OnceLock<Arc<crate::fx::PathSet>>>,
    standalone_cache: Arc<std::sync::OnceLock<ImportResolutionCache>>,
}

impl PreparedVisible {
    pub fn new(paths: crate::fx::PathSet) -> Self {
        Self::from_shared(Arc::new(paths))
    }

    #[doc(hidden)]
    pub fn from_shared(paths: Arc<crate::fx::PathSet>) -> Self {
        Self {
            paths,
            identity: VisibleScopeIdentity::new(),
            normalized: Arc::default(),
            standalone_cache: Arc::default(),
        }
    }

    #[doc(hidden)]
    pub fn identity(&self) -> &VisibleScopeIdentity {
        &self.identity
    }

    #[doc(hidden)]
    pub fn paths_arc(&self) -> &Arc<crate::fx::PathSet> {
        &self.paths
    }

    #[doc(hidden)]
    pub fn normalized_cell(&self) -> &std::sync::OnceLock<Arc<crate::fx::PathSet>> {
        &self.normalized
    }

    pub fn resolver_cache(&self) -> &ImportResolutionCache {
        self.standalone_cache.get_or_init(Default::default)
    }
}

impl FromIterator<PathBuf> for PreparedVisible {
    fn from_iter<T: IntoIterator<Item = PathBuf>>(paths: T) -> Self {
        Self::new(paths.into_iter().collect())
    }
}

impl std::ops::Deref for PreparedVisible {
    type Target = crate::fx::PathSet;
    fn deref(&self) -> &Self::Target {
        &self.paths
    }
}

impl VisiblePathLookup for PreparedVisible {
    fn contains_visible(&self, path: &Path) -> bool {
        self.paths.contains(path)
    }

    fn visible_len(&self) -> usize {
        self.paths.len()
    }

    fn visible_cache_key(&self) -> Vec<PathBuf> {
        self.paths.visible_cache_key()
    }

    fn visit_visible(&self, visit: &mut dyn FnMut(&Path)) {
        self.paths.visit_visible(visit);
    }

    fn scope_identity(&self) -> Option<VisibleScopeIdentity> {
        Some(self.identity.clone())
    }

    fn prepared_projection(&self) -> Option<PreparedVisible> {
        Some(self.clone())
    }

    fn normalized_visible(&self) -> Arc<crate::fx::PathSet> {
        Arc::clone(
            self.normalized
                .get_or_init(|| normalize_visible_paths(&self.paths, canonical_alias)),
        )
    }
}

#[doc(hidden)]
pub fn canonical_alias(path: &Path, normalized: &Path) -> Option<Arc<Path>> {
    // Identity spellings and failures need no extra retained canonical text.
    path.canonicalize()
        .ok()
        .map(|path| normalize_path(&path))
        .filter(|real| real != normalized)
        .map(Arc::<Path>::from)
}

#[doc(hidden)]
pub fn normalize_visible_paths(
    paths: &Arc<crate::fx::PathSet>,
    mut canonical: impl FnMut(&Path, &Path) -> Option<Arc<Path>>,
) -> Arc<crate::fx::PathSet> {
    let mut output = None;
    for path in paths.iter() {
        let normalized = normalize_path(path);
        let alias = canonical(path, &normalized).filter(|path| !paths.contains(path.as_ref()));
        if normalized != *path || alias.is_some() {
            let output = output.get_or_insert_with(|| paths.as_ref().clone());
            if normalized != *path {
                output.remove(path);
                output.insert(normalized);
            }
            if let Some(alias) = alias {
                output.insert(alias.as_ref().to_path_buf());
            }
        }
    }
    output.map_or_else(|| Arc::clone(paths), Arc::new)
}
