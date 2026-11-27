impl WorkspaceMap {
    pub(crate) fn resolve_specifier_from_visible(
        &self,
        specifier: &str,
        visible_files: &dyn VisiblePathLookup,
    ) -> Option<PathBuf> {
        self.resolve_specifier_inner(specifier, Some(visible_files))
    }

    pub(crate) fn recognizes_specifier_from(&self, specifier: &str, importing_file: &Path) -> bool {
        if specifier.starts_with('#') {
            return self.nearest_package(importing_file).is_some_and(|package| {
                package
                    .imports
                    .as_ref()
                    .is_some_and(|imports| resolve_export_subpath(imports, specifier).is_some())
            });
        }
        package_name_and_subpath(specifier)
            .is_some_and(|(name, _)| self.package_by_name(&name).is_some())
    }
}

impl IndexedWorkspaceMap {
    pub(crate) fn from_packages(packages: Vec<WorkspacePackage>) -> Self {
        Self::new(
            WorkspaceMap { packages },
            std::collections::HashSet::new(),
            std::collections::BTreeMap::new(),
        )
    }

    pub(crate) fn with_manifest_dependency_names(
        mut self,
        path: PathBuf,
        mut names: Vec<String>,
    ) -> Self {
        names.sort();
        names.dedup();
        std::sync::Arc::make_mut(&mut self.manifest_dependency_names)
            .insert(normalize_path(&path), names);
        self
    }

    pub(crate) fn shares_indexes_with(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.indexes, &other.indexes)
    }
}
