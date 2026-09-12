impl DepGraph {
    /// Resolve an exported root through the canonical export resolution built
    /// alongside call edges. This is populated for every exported spelling,
    /// including exports with no inbound call site.
    #[inline(never)]
    fn resolve_exported_callable_root(
        &self,
        file: &std::path::Path,
        symbol: &str,
    ) -> Option<Vec<NodeId>> {
        let key = (
            crate::codebase::ts_resolver::normalize_path(file),
            symbol.to_owned(),
        );
        let resolution = self.callable_export_resolutions.get(&key)?;
        match resolution {
            ExportedCallableResolution::Callable(target_file, target_scope, callable_id) => self
                .unique_callable_node(target_file, target_scope, *callable_id)
                .map(|target| vec![target]),
            ExportedCallableResolution::Absent => None,
            ExportedCallableResolution::ExternalModuleExport(_, _)
            | ExportedCallableResolution::Unknown => Some(Vec::new()),
        }
    }

    #[inline(never)]
    fn unique_callable_node(
        &self,
        file: &std::path::Path,
        symbol: &str,
        callable_id: Option<crate::codebase::dependencies::extract::CallableId>,
    ) -> Option<NodeId> {
        let matches = self
            .callable_nodes_by_file
            .get(file)
            .into_iter()
            .flatten()
            .filter(|candidate| {
                matches!(
                    candidate,
                    NodeId::Symbol {
                        symbol: candidate_symbol,
                        callable_id: candidate_id,
                        ..
                    } if candidate_symbol.as_ref() == symbol
                        && callable_id.is_none_or(|id| *candidate_id == Some(id))
                )
            })
            .cloned();
        let mut matches = matches;
        let node = matches.next()?;
        matches.next().is_none().then_some(node)
    }
}
