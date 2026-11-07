impl DepGraph {
    fn imported_selector_target(&self, file: &std::path::Path, selector: &str) -> Option<&NodeId> {
        let edges = self.dependencies_of_node(&NodeId::symbol(file, selector))?;
        let mut value = None;
        for (target, kind) in edges {
            if *kind == EdgeKind::SymbolImportUnknown {
                return None;
            }
            if *kind != EdgeKind::SymbolImport {
                continue;
            }
            if value.is_some_and(|value| value != target) {
                return None;
            }
            value = Some(target);
        }
        value
    }

    /// Canonical live const binding selected by one retained runtime import.
    pub(crate) fn imported_value(
        &self,
        file: &std::path::Path,
        selector: &str,
    ) -> Option<(std::path::PathBuf, String)> {
        match self.imported_selector_target(file, selector)? {
            NodeId::Symbol {
                file,
                symbol,
                callable_id: None,
            } => Some((file.to_path_buf(), symbol.to_string())),
            _ => None,
        }
    }

    pub(crate) fn imported_namespace(
        &self,
        file: &std::path::Path,
        selector: &str,
    ) -> Option<std::path::PathBuf> {
        match self.imported_selector_target(file, selector)? {
            NodeId::File(namespace) => Some(namespace.to_path_buf()),
            _ => None,
        }
    }

    pub(crate) fn namespace_exports_value(
        &self,
        namespace: &std::path::Path,
        producer: &std::path::Path,
        local: &str,
    ) -> bool {
        let target = NodeId::symbol(producer, local);
        let mut pending = vec![NodeId::file(namespace)];
        let mut visited = fx_set();
        while let Some(namespace) = pending.pop() {
            if !visited.insert(namespace.clone()) {
                continue;
            }
            for (export, kind) in self.dependencies_of_node(&namespace).into_iter().flatten() {
                if *kind != EdgeKind::SymbolReexport {
                    continue;
                }
                if export == &target {
                    return true;
                }
                if matches!(export, NodeId::File(_)) {
                    pending.push(export.clone());
                }
            }
        }
        false
    }
}
