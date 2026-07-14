#[derive(Debug, Default)]
struct WorkspaceIndexes {
    by_name: std::collections::BTreeMap<String, usize>,
    by_dir: std::collections::BTreeMap<PathBuf, usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct IndexedWorkspaceMap {
    workspace: std::sync::Arc<WorkspaceMap>,
    indexes: std::sync::Arc<WorkspaceIndexes>,
    root_dependency_names: std::sync::Arc<std::collections::HashSet<String>>,
    manifest_dependency_names: std::sync::Arc<std::collections::BTreeMap<PathBuf, Vec<String>>>,
}

impl Default for IndexedWorkspaceMap {
    fn default() -> Self {
        Self::new(
            WorkspaceMap::default(),
            std::collections::HashSet::new(),
            std::collections::BTreeMap::new(),
        )
    }
}

impl std::ops::Deref for IndexedWorkspaceMap {
    type Target = WorkspaceMap;
    fn deref(&self) -> &Self::Target {
        self.workspace.as_ref()
    }
}

impl IndexedWorkspaceMap {    fn new(
        workspace: WorkspaceMap,
        root_dependency_names: std::collections::HashSet<String>,
        manifest_dependency_names: std::collections::BTreeMap<PathBuf, Vec<String>>,
    ) -> Self {
        let mut indexes = WorkspaceIndexes::default();
        for (index, package) in workspace.packages.iter().enumerate() {
            indexes.by_name.entry(package.name.clone()).or_insert(index);
            indexes
                .by_dir
                .entry(normalize_path(&package.dir))
                .or_insert(index);
        }
        Self {
            workspace: std::sync::Arc::new(workspace),
            indexes: std::sync::Arc::new(indexes),
            root_dependency_names: std::sync::Arc::new(root_dependency_names),
            manifest_dependency_names: std::sync::Arc::new(manifest_dependency_names),
        }
    }

    pub(crate) fn package_by_name(&self, name: &str) -> Option<&WorkspacePackage> {
        self.indexes
            .by_name
            .get(name)
            .and_then(|index| self.packages.get(*index))
    }

    pub(crate) fn package_by_dir(&self, dir: &Path) -> Option<&WorkspacePackage> {
        self.indexes
            .by_dir
            .get(&normalize_path(dir))
            .and_then(|index| self.packages.get(*index))
    }

    pub(crate) fn root_dependency_names(&self) -> &std::collections::HashSet<String> {
        self.root_dependency_names.as_ref()
    }

    pub(crate) fn manifest_dependency_names(&self, path: &Path) -> Option<&[String]> {
        self.manifest_dependency_names
            .get(&normalize_path(path))
            .map(Vec::as_slice)
    }

    pub(crate) fn resolve_specifier_from_visible(
        &self,
        specifier: &str,
        visible_files: &std::collections::HashSet<PathBuf>,
    ) -> Option<PathBuf> {
        self.resolve_specifier_inner(specifier, None, Some(visible_files))
    }

    pub(crate) fn resolve_specifier_from_file_visible(
        &self,
        specifier: &str,
        importing_file: &Path,
        visible_files: &std::collections::HashSet<PathBuf>,
    ) -> Option<PathBuf> {
        self.resolve_specifier_inner(specifier, Some(importing_file), Some(visible_files))
    }

    pub(crate) fn recognizes_specifier_from(&self, specifier: &str, importing_file: &Path) -> bool {
        if specifier.starts_with("#") {
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

    fn resolve_specifier_inner(
        &self,
        specifier: &str,
        importing_file: Option<&Path>,
        visible_files: Option<&std::collections::HashSet<PathBuf>>,
    ) -> Option<PathBuf> {
        if specifier.starts_with("#") {
            return self
                .nearest_package(importing_file?)
                .and_then(|package| package.resolve_import(specifier, visible_files));
        }
        let (name, subpath) = package_name_and_subpath(specifier)?;
        let package = self.package_by_name(&name)?;
        if let Some(subpath) = subpath {
            package.resolve_subpath(&subpath, visible_files)
        } else {
            package.entry.clone().filter(|entry| {
                visible_files.is_none_or(|visible| visible.contains(&normalize_path(entry)))
            })
        }
    }

    fn nearest_package(&self, importing_file: &Path) -> Option<&WorkspacePackage> {
        normalize_path(importing_file)
            .ancestors()
            .find_map(|candidate| self.package_by_dir(candidate))
    }
}
