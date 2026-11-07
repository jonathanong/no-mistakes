impl DepGraph {
    pub(crate) fn matches_module_export(
        &self,
        module: &str,
        export: &str,
        site: &ResolvedCallSite,
    ) -> bool {
        if site.has_unknown_target() || site.targets().len() == 0 {
            return false;
        }
        site.targets().all(|candidate| {
            if matches!(candidate.target, KnownCallTargetRef::ModuleExport { specifier, export_path, .. }
                if specifier == module && export_path == export)
            {
                return true;
            }
            self.module_files.get(module).into_iter().flatten().any(|file| {
                match self.callable_export_resolutions.get(&(file.clone(), export.to_string())) {
                    Some(ExportedCallableResolution::Callable(producer, scope, id)) => {
                        let target = id.map_or_else(
                            || NodeId::symbol(producer, scope.clone()),
                            |id| NodeId::callable(producer, scope.clone(), id),
                        );
                        candidate.node == Some(&target)
                    }
                    Some(ExportedCallableResolution::ExternalModuleExport(origin, name)) => {
                        matches!(candidate.target, KnownCallTargetRef::ModuleExport { specifier, export_path, .. }
                            if specifier == origin && export_path == name)
                    }
                    _ => false,
                }
            })
        })
    }
}
