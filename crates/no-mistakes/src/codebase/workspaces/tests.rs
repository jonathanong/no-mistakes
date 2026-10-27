use super::*;
use tempfile::TempDir;

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

mod extra;
mod pnpm_include;
mod resolution;
mod visibility;

fn workspace_case(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join(name)
        .join("fixture")
}

// Child modules still call write(); copy saved bytes instead of creating them inline.
fn write(path: &Path, content: &str) {
    ensure_parent(path);
    std::fs::copy(workspace_byte_blob(content), path).unwrap();
}

fn ensure_parent(path: &Path) {
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return;
    };
    if parent.exists() {
        return;
    }
    ensure_parent(parent);
    std::fs::create_dir(parent).unwrap();
}

fn workspace_byte_blob(content: &str) -> PathBuf {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase/workspaces-byte-blobs")
        .join(format!("{hash:016x}"));
    assert!(
        saved.is_file(),
        "missing workspace byte blob for {content:?}"
    );
    saved
}

fn resolve_entry(dir: &Path, pkg: &PackageJson) -> Option<PathBuf> {
    resolve_entry_with_visibility(dir, pkg, None)
}

// ── load with no package.json ─────────────────────────────────────────

#[test]
fn no_package_json_returns_empty() {
    let dir = TempDir::new().unwrap();
    let map = load(dir.path()).unwrap();
    assert!(map.packages.is_empty());
}

#[test]
fn invalid_workspace_glob_returns_no_dirs() {
    let map = load(&workspace_case("workspaces-invalid-glob")).unwrap();

    assert!(map.packages.is_empty());
}

// ── load with workspaces as array ─────────────────────────────────────

#[test]
fn loads_workspace_array() {
    let root = workspace_case("workspaces-array");

    let map = load(&root).unwrap();
    assert_eq!(map.packages.len(), 1);
    assert_eq!(map.packages[0].name, "@x/api");
    assert!(map.packages[0].entry.is_some());
}

// ── load with workspaces as object ────────────────────────────────────

#[test]
fn loads_workspace_object_form() {
    let root = workspace_case("workspaces-object-form");

    let map = load(&root).unwrap();
    assert_eq!(map.packages.len(), 1);
    assert_eq!(map.packages[0].name, "@x/web");
}

#[test]
fn loads_pnpm_workspace_yaml_when_package_json_has_no_workspaces() {
    let root = workspace_case("workspaces-pnpm-yaml");

    let map = load(&root).unwrap();
    assert_eq!(map.packages.len(), 1);
    assert_eq!(map.packages[0].name, "@x/api");
}

#[test]
fn pnpm_workspace_yaml_takes_precedence_over_package_json_workspaces() {
    let root = workspace_case("workspaces-pnpm-precedence");

    let map = load(&root).unwrap();
    assert_eq!(map.packages.len(), 1);
    assert_eq!(map.packages[0].name, "@x/pnpm");
}

#[test]
fn pnpm_workspace_without_packages_includes_direct_subdirectories() {
    let root = workspace_case("workspaces-pnpm-direct-subdirs");

    let map = load(&root).unwrap();
    assert_eq!(map.packages.len(), 1);
    assert_eq!(map.packages[0].name, "@x/api");
}

#[test]
fn pnpm_workspace_exclusion_globs_remove_loaded_packages() {
    let root = workspace_case("workspaces-pnpm-exclusion");

    let map = load(&root).unwrap();
    assert_eq!(map.packages.len(), 1);
    assert_eq!(map.packages[0].name, "@x/foo");
}

// ── Workspace package entries ─────────────────────────────────────────

#[test]
fn resolve_package_finds_by_name() {
    let dir = workspace_case("workspaces-resolve-package-by-name");
    let entry = dir.join("src/index.mts");
    let map = WorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@x/api".to_string(),
        dir,
        entry: Some(entry.clone()),
        exports: None,
        imports: None,
    }]);
    assert_eq!(map.resolve_package("@x/api"), Some(&entry));
}

#[test]
fn package_indexes_fall_back_safely_after_public_package_mutation() {
    let mut map = WorkspaceMap::from_packages(vec![
        WorkspacePackage {
            name: "@x/one".to_string(),
            dir: PathBuf::from("/repo/one"),
            entry: None,
            exports: None,
            imports: None,
        },
        WorkspacePackage {
            name: "@x/two".to_string(),
            dir: PathBuf::from("/repo/two"),
            entry: None,
            exports: None,
            imports: None,
        },
    ]);

    map.packages.swap(0, 1);
    map.packages.push(WorkspacePackage {
        name: "@x/three".to_string(),
        dir: PathBuf::from("/repo/three"),
        entry: None,
        exports: None,
        imports: None,
    });

    assert_eq!(
        map.package_by_name("@x/one").unwrap().dir,
        PathBuf::from("/repo/one")
    );
    assert_eq!(
        map.package_by_dir(Path::new("/repo/two")).unwrap().name,
        "@x/two"
    );
    assert_eq!(
        map.package_by_name("@x/three").unwrap().dir,
        PathBuf::from("/repo/three")
    );
}

#[test]
fn resolve_package_missing_returns_none() {
    let map = WorkspaceMap::default();
    assert!(map.resolve_package("@x/missing").is_none());
}

#[test]
fn package_indexes_preserve_first_match_and_nearest_directory_behavior() {
    let packages = vec![
        WorkspacePackage {
            name: "@x/shared".to_string(),
            dir: PathBuf::from("/repo/packages/outer"),
            entry: Some(PathBuf::from("/repo/packages/outer/index.mts")),
            exports: None,
            imports: None,
        },
        WorkspacePackage {
            name: "@x/shared".to_string(),
            dir: PathBuf::from("/repo/packages/outer/nested"),
            entry: Some(PathBuf::from("/repo/packages/outer/nested/index.mts")),
            exports: None,
            imports: None,
        },
    ];
    let map = WorkspaceMap::from_packages(packages);

    assert_eq!(
        map.resolve_package("@x/shared"),
        Some(&PathBuf::from("/repo/packages/outer/index.mts"))
    );
    assert_eq!(
        map.nearest_package(Path::new("/repo/packages/outer/nested/src/feature.mts"))
            .map(|package| package.dir.as_path()),
        Some(Path::new("/repo/packages/outer/nested"))
    );
    assert_eq!(
        map.package_by_dir(Path::new("/repo/packages/outer/ignored/../nested"))
            .map(|package| package.dir.as_path()),
        Some(Path::new("/repo/packages/outer/nested"))
    );
}

#[test]
fn loaded_workspace_reuses_root_dependency_metadata() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/codebase-intel/fixture");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);

    let map = load_indexed_from_files(&root, &files).unwrap();

    assert_eq!(
        map.root_dependency_names(),
        &std::collections::HashSet::from(["@x/web".to_string()])
    );
}

#[test]
fn resolve_specifier_rejects_relative_and_missing_packages() {
    let map = WorkspaceMap::default();

    assert_eq!(map.resolve_specifier("./local"), None);
    assert_eq!(map.resolve_specifier("/abs"), None);
    assert_eq!(map.resolve_specifier("@missing/pkg/subpath"), None);
}

#[test]
fn resolve_specifier_rejects_unprefixed_subpath_without_exports() {
    let dir = TempDir::new().unwrap();
    let map = WorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@x/api".to_string(),
        dir: dir.path().to_path_buf(),
        entry: None,
        exports: None,
        imports: None,
    }]);

    assert_eq!(map.resolve_specifier("@x/api"), None);
    assert_eq!(map.packages[0].resolve_subpath("src/public", None), None);
}

// ── entry resolution order ────────────────────────────────────────────
