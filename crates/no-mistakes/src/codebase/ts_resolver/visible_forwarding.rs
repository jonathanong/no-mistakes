macro_rules! forward_visible_lookup {
    () => {
        fn contains_visible(&self, path: &Path) -> bool {
            (**self).contains_visible(path)
        }

        fn visible_len(&self) -> usize {
            (**self).visible_len()
        }

        fn visit_visible(&self, visit: &mut dyn FnMut(&Path)) {
            (**self).visit_visible(visit);
        }
        fn scope_identity(&self) -> Option<VisibleScopeIdentity> {
            (**self).scope_identity()
        }
        fn prepared_projection(
            &self,
        ) -> Option<crate::codebase::analysis_session::PreparedVisible> {
            (**self).prepared_projection()
        }

        fn visible_cache_key(&self) -> Vec<PathBuf> {
            (**self).visible_cache_key()
        }

        fn normalized_visible(&self) -> Arc<crate::fx::PathSet> {
            (**self).normalized_visible()
        }

        fn visible_alias(&self, path: &Path) -> Option<PathBuf> {
            (**self).visible_alias(path)
        }
    };
}

impl<T: VisiblePathLookup + ?Sized> VisiblePathLookup for &T {
    forward_visible_lookup!();
}

impl<T: VisiblePathLookup + ?Sized> VisiblePathLookup for std::sync::Arc<T> {
    forward_visible_lookup!();
}
