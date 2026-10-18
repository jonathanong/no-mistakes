/// Membership test for a frozen visible-path universe.
///
/// Graph construction uses [`crate::codebase::dependencies::graph::GraphFiles`]
/// so lookups binary-search interned paths instead of hashing cloned PathBufs.
/// HashSet remains for callers that already own a path set.
pub(crate) trait VisiblePathLookup: Send + Sync {
    fn contains_visible(&self, path: &Path) -> bool;

    fn visible_len(&self) -> usize;

    /// Paths that distinguish one resolver cache scope from another.
    fn visible_cache_key(&self) -> Vec<PathBuf>;

    /// Visit exact members without sorting or cloning a repository-sized key.
    fn visit_visible(&self, visit: &mut dyn FnMut(&Path)) {
        for path in self.visible_cache_key() {
            visit(&path);
        }
    }

    /// Reserved for prepared exact membership and default alias semantics.
    fn scope_identity(&self) -> Option<VisibleScopeIdentity> {
        None
    }

    /// Reuse storage already frozen by a request owner instead of copying paths.
    fn prepared_projection(&self) -> Option<crate::codebase::analysis_session::PreparedVisible> {
        None
    }

    /// Normalized plus canonical aliases used by scoped import resolution.
    ///
    /// Graph-backed lookups override this to reuse one request-scoped Arc;
    /// standalone callers retain the same owned projection semantics.
    fn normalized_visible(&self) -> Arc<crate::fx::PathSet> {
        Arc::new(normalized_visible_path_set(self.visible_cache_key()))
    }

    /// Preferred visible spelling for `path`, including symlink aliases.
    fn visible_alias(&self, path: &Path) -> Option<PathBuf> {
        self.contains_visible(path)
            .then(|| crate::codebase::ts_resolver::normalize_path(path))
    }
}

pub(crate) fn normalized_visible_path_set(
    paths: impl IntoIterator<Item = PathBuf>,
) -> crate::fx::PathSet {
    paths
        .into_iter()
        .flat_map(|path| {
            let normalized = crate::codebase::ts_resolver::normalize_path(&path);
            path.canonicalize().ok().map_or_else(
                || vec![normalized.clone()],
                |real| {
                    vec![
                        normalized.clone(),
                        crate::codebase::ts_resolver::normalize_path(&real),
                    ]
                },
            )
        })
        .collect()
}

impl VisiblePathLookup for HashSet<PathBuf> {
    fn contains_visible(&self, path: &Path) -> bool {
        self.contains(path)
    }

    fn visible_len(&self) -> usize {
        self.len()
    }

    fn visit_visible(&self, visit: &mut dyn FnMut(&Path)) {
        for path in self {
            visit(path);
        }
    }

    fn visible_cache_key(&self) -> Vec<PathBuf> {
        let mut paths: Vec<_> = self.iter().cloned().collect();
        paths.sort();
        paths
    }
}

impl VisiblePathLookup for crate::fx::PathSet {
    fn contains_visible(&self, path: &Path) -> bool {
        self.contains(path)
    }

    fn visible_len(&self) -> usize {
        self.len()
    }

    fn visit_visible(&self, visit: &mut dyn FnMut(&Path)) {
        for path in self {
            visit(path);
        }
    }

    fn visible_cache_key(&self) -> Vec<PathBuf> {
        let mut paths: Vec<_> = self.iter().cloned().collect();
        paths.sort();
        paths
    }
}

#[cfg(test)]
#[path = "visible_tests.rs"]
mod visible_tests;
