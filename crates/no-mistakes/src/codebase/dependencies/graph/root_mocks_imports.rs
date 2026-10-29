impl DepGraph {
    fn mock_factory_imports_are_full(
        &self,
        path: &Path,
        span: &crate::codebase::ts_source::module_facts::ModuleSpan,
        target: &Path,
        module: &crate::codebase::ts_source::module_facts::TypeScriptModuleFacts,
        resolver: &dyn ImportResolution,
        graph_files: &GraphFiles,
    ) -> bool {
        let referenced: FxHashSet<_> = module
            .bindings
            .iter()
            .filter(|binding| {
                binding.imported
                    && binding.references.iter().any(|reference| {
                        reference.runtime
                            && span.start <= reference.span.start
                            && reference.span.start < span.end
                    })
            })
            .map(|binding| binding.id)
            .collect();
        module
            .imports
            .iter()
            .filter(|import| {
                import
                    .bindings
                    .iter()
                    .any(|binding| referenced.contains(&binding.binding_id))
            })
            .all(|import| {
                resolver
                    .resolve(&import.specifier, path)
                    .and_then(|file| graph_files.visible_path(&file))
                    .is_some_and(|file| !self.import_reaches_mock(file, target))
            })
    }

    fn import_reaches_mock(&self, file: &Path, target: &Path) -> bool {
        let mut seen = fx_set();
        let mut queue = VecDeque::from([NodeId::file(file)]);
        while let Some(node) = queue.pop_front() {
            if node.as_file() == Some(target) {
                return true;
            }
            if !seen.insert(node.clone()) {
                continue;
            }
            let Some(neighbors) = self.edges.forward().get(&node) else {
                continue;
            };
            for (neighbor, kind) in &neighbors.neighbors {
                if matches!(
                    kind,
                    EdgeKind::Import
                        | EdgeKind::WorkspaceImport
                        | EdgeKind::Require
                        | EdgeKind::DynamicImport
                        | EdgeKind::ConditionalDynamicImport
                ) {
                    queue.push_back(neighbor.clone());
                }
            }
        }
        false
    }
}
