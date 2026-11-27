#[derive(Clone)]
pub(crate) struct ServerRouteFactFilter {
    glob: GlobSet,
    test_filter: Option<crate::codebase::testing_analysis::test_filter::TestFileFilter>,
}

impl std::fmt::Debug for ServerRouteFactFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerRouteFactFilter")
            .field("test_filter", &self.test_filter.is_some())
            .finish_non_exhaustive()
    }
}

impl ServerRouteFactFilter {
    pub(crate) fn new(
        glob: GlobSet,
        test_filter: Option<crate::codebase::testing_analysis::test_filter::TestFileFilter>,
    ) -> Self {
        Self { glob, test_filter }
    }

    fn is_match(&self, root: &Path, path: &Path) -> bool {
        path.strip_prefix(root)
            .is_ok_and(|rel| self.glob.is_match(rel))
            && !self
                .test_filter
                .as_ref()
                .is_some_and(|filter| filter.is_match(root, path))
    }
}

impl TsFactContext {
    pub fn set_visible_files(&mut self, files: impl IntoIterator<Item = PathBuf>) {
        self.set_visible_file_set(
            files
                .into_iter()
                .map(|path| crate::codebase::ts_resolver::normalize_path(&path))
                .collect(),
        );
    }

    /// Install an already-normalized visible set without cloning it again.
    ///
    /// An empty set stays `Some` so scoped analyses keep an explicit empty
    /// universe instead of falling back to unrestricted filesystem resolution.
    pub fn set_visible_file_set(&mut self, files: crate::fx::PathSet) {
        self.visible_files = Some(Arc::new(files));
    }
}

impl TsFactContext {
    #[doc(hidden)]
    pub fn set_fixture_reads(&mut self, reads: Option<&Arc<FixtureReadIndex>>) {
        self.fixture_reads = reads.cloned();
    }
}
