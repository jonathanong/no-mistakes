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
    pub fn set_visible_file_set(&mut self, files: crate::fx::PathSet) {
        if files.is_empty() {
            self.visible_files = None;
            return;
        }
        self.visible_files = Some(Arc::new(files));
    }

    /// Share a previously built visible set across fact-context consumers.
    pub fn share_visible_files(&mut self, files: Arc<crate::fx::PathSet>) {
        self.visible_files = Some(files);
    }
}
