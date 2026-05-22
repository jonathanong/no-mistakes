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
        let (name, subpath) = package_name_and_subpath(specifier)?;
        let package = self.packages.iter().find(|p| p.name == name)?;
        if subpath.is_none() {
            return package.entry.clone();
        }
        package.resolve_subpath(subpath.as_deref()?)
    }

    /// Resolve a package specifier from the importing file's package context.
    pub fn resolve_specifier_from(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf> {
        if specifier.starts_with('#') {
            let package = self.nearest_package(importing_file)?;
            return package.resolve_import(specifier);
        }
        self.resolve_specifier(specifier)
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
    fn resolve_subpath(&self, subpath: &str) -> Option<PathBuf> {
        if let Some(exports) = &self.exports {
            let target = resolve_export_subpath(exports, subpath)?;
            return try_resolve(&normalize_path(&self.dir.join(target)));
        }

        let relative = subpath.strip_prefix("./")?;
        let candidate = normalize_path(&self.dir.join(relative));
        try_resolve(&candidate)
    }

    fn resolve_import(&self, specifier: &str) -> Option<PathBuf> {
        let imports = self.imports.as_ref()?;
        let target = resolve_export_subpath(imports, specifier)?;
        try_resolve(&normalize_path(&self.dir.join(target)))
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
pub fn load(root: &Path) -> Result<WorkspaceMap> {
    let workspace_globs = load_workspace_globs(root)?;
    let dirs = expand_workspace_globs(root, &workspace_globs);
    load_packages_from_dirs(dirs)
}

pub fn load_from_files(root: &Path, files: &[PathBuf]) -> Result<WorkspaceMap> {
    let workspace_globs = load_workspace_globs(root)?;
    let dirs = expand_workspace_globs_from_files(root, &workspace_globs, files);
    load_packages_from_dirs(dirs)
}

fn load_packages_from_dirs(dirs: Vec<PathBuf>) -> Result<WorkspaceMap> {
    let mut packages = Vec::new();
    for dir in dirs {
        if let Some(pkg) = load_package(&dir)? {
            packages.push(pkg);
        }
    }

    Ok(WorkspaceMap { packages })
}
