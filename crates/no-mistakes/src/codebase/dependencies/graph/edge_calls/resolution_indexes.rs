impl CallableResolutionIndexes {
    fn for_value_imports(&self) -> Self {
        Self {
            files: self.files.clone(),
            namespace_roots: self.namespace_roots.clone(),
            value_imports: true,
            external_namespaces: self.external_namespaces,
            exports: Default::default(),
            export_prerequisites: Default::default(),
        }
    }

    fn file(
        &self,
        facts: &dyn TsFactLookup,
        path: &std::path::Path,
    ) -> Option<std::sync::Arc<CallableFileIndex>> {
        if let Some(index) = self.files.get(path) {
            return Some(index.clone());
        }
        let file = facts.get_ts_facts(path)?;
        let index = std::sync::Arc::new(CallableFileIndex::from_facts(file));
        self.files.insert(path.to_path_buf(), index.clone());
        Some(index)
    }
}
