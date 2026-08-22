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
