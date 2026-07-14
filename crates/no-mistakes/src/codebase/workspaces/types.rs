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
#[derive(Debug, Default, Clone)]
pub struct WorkspaceMap {
    pub packages: Vec<WorkspacePackage>,
}

impl WorkspaceMap {
    /// Resolve a workspace package name to its entry file.
    pub fn resolve_package(&self, name: &str) -> Option<&PathBuf> {
        self.packages
            .iter()
            .find(|package| package.name == name)
            .and_then(|package| package.entry.as_ref())
    }

    /// Resolve a bare workspace import specifier to the package entry or an exported subpath.
    pub fn resolve_specifier(&self, specifier: &str) -> Option<PathBuf> {
        self.resolve_specifier_inner(specifier, None)
    }

    pub(crate) fn resolve_specifier_from_visible(
        &self,
        specifier: &str,
        visible_files: &std::collections::HashSet<PathBuf>,
    ) -> Option<PathBuf> {
        self.resolve_specifier_inner(specifier, Some(visible_files))
    }

    fn resolve_specifier_inner(
        &self,
        specifier: &str,
        visible_files: Option<&std::collections::HashSet<PathBuf>>,
    ) -> Option<PathBuf> {
        let (name, subpath) = package_name_and_subpath(specifier)?;
        let package = self.packages.iter().find(|p| p.name == name)?;
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

    /// Whether a specifier belongs to this workspace even when its resolved
    /// target is absent from the visible-file universe. Graph builders use
    /// this distinction to drop ignored workspace targets instead of
    /// misclassifying them as external package modules.
    pub(crate) fn recognizes_specifier_from(
        &self,
        specifier: &str,
        importing_file: &Path,
    ) -> bool {
        if specifier.starts_with('#') {
            return self.nearest_package(importing_file).is_some_and(|package| {
                package.imports.as_ref().is_some_and(|imports| {
                    resolve_export_subpath(imports, specifier).is_some()
                })
            });
        }
        package_name_and_subpath(specifier).is_some_and(|(name, _)| {
            self.packages.iter().any(|package| package.name == name)
        })
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
        self.packages
            .iter()
            .filter(|package| importing_file.starts_with(&package.dir))
            .max_by_key(|package| package.dir.components().count())
    }
}

impl WorkspacePackage {
    #[inline(never)]
    fn resolve_subpath(
        &self,
        subpath: &str,
        visible_files: Option<&std::collections::HashSet<PathBuf>>,
    ) -> Option<PathBuf> {
        if let Some(exports) = &self.exports {
            let target = resolve_export_subpath(exports, subpath)?;
            return resolve_workspace_path(&normalize_path(&self.dir.join(target)), visible_files);
        }

        let relative = subpath.strip_prefix("./")?;
        let candidate = normalize_path(&self.dir.join(relative));
        resolve_workspace_path(&candidate, visible_files)
    }

    fn resolve_import(
        &self,
        specifier: &str,
        visible_files: Option<&std::collections::HashSet<PathBuf>>,
    ) -> Option<PathBuf> {
        let imports = self.imports.as_ref()?;
        let target = resolve_export_subpath(imports, specifier)?;
        resolve_workspace_path(&normalize_path(&self.dir.join(target)), visible_files)
    }
}

#[derive(Deserialize, Default)]
struct PackageJson {
    name: Option<String>,
    workspaces: Option<WorkspacesField>,
    main: Option<String>,
    module: Option<String>,
    exports: Option<serde_json::Value>,
    imports: Option<serde_json::Value>,
    types: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum WorkspacesField {
    Array(Vec<String>),
    Object { packages: Vec<String> },
}

#[derive(Deserialize, Default)]
struct PnpmWorkspace {
    packages: Option<Vec<String>>,
}

/// Load the workspace map from `root/package.json` or `root/pnpm-workspace.yaml`.
///
/// Returns an empty map if neither file declares workspaces.
///
/// Derives package directories from the shared ignore-aware candidate list.
/// Callers that already have a discovered file list on hand should use
/// [`load_from_files`] directly to avoid repeating discovery.
pub fn load(root: &Path) -> Result<WorkspaceMap> {
    let files = crate::codebase::ts_source::discover_visible_paths(root);
    load_from_files(root, &files)
}

pub fn load_from_files(root: &Path, files: &[PathBuf]) -> Result<WorkspaceMap> {
    let workspace_globs = load_workspace_globs_from_files(root, files)?;
    let dirs = expand_workspace_globs_from_files(root, &workspace_globs, files);
    let visible: std::collections::HashSet<PathBuf> = files
        .iter()
        .map(|path| normalize_path(path))
        .collect();
    load_packages_from_dirs(dirs, &visible)
}

fn load_packages_from_dirs(
    dirs: Vec<PathBuf>,
    visible_files: &std::collections::HashSet<PathBuf>,
) -> Result<WorkspaceMap> {
    let mut packages = Vec::new();
    for dir in dirs {
        if let Some(pkg) = load_package(&dir, Some(visible_files))? {
            packages.push(pkg);
        }
    }

    Ok(WorkspaceMap { packages })
}
