#[derive(Debug, Clone)]
pub struct WorkspacePackage {
    /// The `name` field from the package's `package.json`.
    pub name: String,
    /// Absolute path to the package directory.
    pub dir: PathBuf,
    /// Resolved absolute path to the package entry file, if any.
    pub entry: Option<PathBuf>,
    /// Raw `exports` field from package.json, used for exact and pattern subpath exports.
    pub exports: Option<serde_json::Value>,
    /// Raw `imports` field from package.json, used for local `#...` imports.
    pub imports: Option<serde_json::Value>,
}

/// All NPM workspace packages resolved from a root `package.json`.
#[derive(Debug, Clone)]
pub struct WorkspaceMap {
    pub packages: Vec<WorkspacePackage>,
}

impl Default for WorkspaceMap {
    fn default() -> Self {
        Self::from_packages(Vec::new())
    }
}

impl WorkspaceMap {
    /// Build a workspace map and its deterministic package lookup indexes.
    pub fn from_packages(packages: Vec<WorkspacePackage>) -> Self {
        Self { packages }
    }

    /// Return the package with the requested workspace name.
    pub fn package_by_name(&self, name: &str) -> Option<&WorkspacePackage> {
        self.packages.iter().find(|package| package.name == name)
    }

    /// Return the package rooted at the requested directory.
    pub fn package_by_dir(&self, dir: &Path) -> Option<&WorkspacePackage> {
        let dir = normalize_path(dir);
        self.packages
            .iter()
            .find(|package| normalize_path(&package.dir) == dir)
    }

    /// Resolve a workspace package name to its entry file.
    pub fn resolve_package(&self, name: &str) -> Option<&PathBuf> {
        self.package_by_name(name)
            .and_then(|package| package.entry.as_ref())
    }

    /// Resolve a bare workspace import specifier to the package entry or an exported subpath.
    pub fn resolve_specifier(&self, specifier: &str) -> Option<PathBuf> {
        self.resolve_specifier_inner(specifier, None)
    }
    fn resolve_specifier_inner(
        &self,
        specifier: &str,
        visible_files: Option<&std::collections::HashSet<PathBuf>>,
    ) -> Option<PathBuf> {
        let (name, subpath) = package_name_and_subpath(specifier)?;
        let package = self.package_by_name(&name)?;
        if subpath.is_none() {
            return package.entry.clone().filter(|entry| {
                visible_files.is_none_or(|visible| visible.contains(&normalize_path(entry)))
            });
        }
        package.resolve_subpath(subpath.as_deref()?, visible_files)
    }

    /// Resolve a package specifier from the importing file's package context.
    pub fn resolve_specifier_from(
        &self,
        specifier: &str,
        importing_file: &Path,
    ) -> Option<PathBuf> {
        self.resolve_specifier_from_inner(specifier, importing_file, None)
    }

    pub(crate) fn resolve_specifier_from_file_visible(
        &self,
        specifier: &str,
        importing_file: &Path,
        visible_files: &std::collections::HashSet<PathBuf>,
    ) -> Option<PathBuf> {
        self.resolve_specifier_from_inner(specifier, importing_file, Some(visible_files))
    }
    fn resolve_specifier_from_inner(
        &self,
        specifier: &str,
        importing_file: &Path,
        visible_files: Option<&std::collections::HashSet<PathBuf>>,
    ) -> Option<PathBuf> {
        if specifier.starts_with('#') {
            let package = self.nearest_package(importing_file)?;
            return package.resolve_import(specifier, visible_files);
        }
        self.resolve_specifier_inner(specifier, visible_files)
    }

    fn nearest_package(&self, importing_file: &Path) -> Option<&WorkspacePackage> {
        let importing_file = normalize_path(importing_file);
        importing_file
            .ancestors()
            .find_map(|candidate| self.package_by_dir(candidate))
    }
}
