impl DepGraph {
    fn root_automock_manual_factories(
        &self,
        inputs: &GraphEdgeBuildInputs<'_>,
        target: &Path,
    ) -> Option<Vec<NodeId>> {
        let base = target.parent()?.join("__mocks__").join(target.file_name()?);
        let candidates: Vec<_> = crate::codebase::ts_source::TS_JS_EXTENSIONS
            .iter()
            .map(|extension| base.with_extension(extension))
            .filter(|path| inputs.graph_files.contains_visible(path))
            .map(CallRoot::File)
            .collect();
        (!candidates.is_empty()).then(|| self.expand_call_roots(&candidates))
    }
}
