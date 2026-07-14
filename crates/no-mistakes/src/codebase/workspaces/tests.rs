use super::*;
use tempfile::TempDir;

impl WorkspaceMap {
    pub(crate) fn resolve_specifier_from_visible(
        &self,
        specifier: &str,
        visible_files: &std::collections::HashSet<PathBuf>,
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
mod resolution;
mod visibility;

fn write(path: &Path, content: &str) {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    std::fs::write(path, content).unwrap();
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
    let dir = TempDir::new().unwrap();
    write(&dir.path().join("package.json"), r#"{"workspaces": ["["]}"#);

    let map = load(dir.path()).unwrap();

    assert!(map.packages.is_empty());
}

// ── load with workspaces as array ─────────────────────────────────────

#[test]
fn loads_workspace_array() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    write(
        &root.join("package.json"),
        r#"{"workspaces": ["packages/*"]}"#,
    );
    write(
        &root.join("packages/api/package.json"),
        r#"{"name": "@x/api", "main": "src/index.mts"}"#,
    );
    write(&root.join("packages/api/src/index.mts"), "export {};");

    let map = load(root).unwrap();
    assert_eq!(map.packages.len(), 1);
    assert_eq!(map.packages[0].name, "@x/api");
    assert!(map.packages[0].entry.is_some());
}

// ── load with workspaces as object ────────────────────────────────────

#[test]
fn loads_workspace_object_form() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    write(
        &root.join("package.json"),
        r#"{"workspaces": {"packages": ["packages/*"]}}"#,
    );
    write(
        &root.join("packages/web/package.json"),
        r#"{"name": "@x/web", "main": "src/index.tsx"}"#,
    );
    write(&root.join("packages/web/src/index.tsx"), "export {};");

    let map = load(root).unwrap();
    assert_eq!(map.packages.len(), 1);
    assert_eq!(map.packages[0].name, "@x/web");
}

#[test]
fn loads_pnpm_workspace_yaml_when_package_json_has_no_workspaces() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    write(&root.join("package.json"), r#"{"name": "root"}"#);
    write(
        &root.join("pnpm-workspace.yaml"),
        "packages:\n  - packages/*\n",
    );
    write(
        &root.join("packages/api/package.json"),
        r#"{"name": "@x/api", "main": "src/index.mts"}"#,
    );
    write(&root.join("packages/api/src/index.mts"), "export {};");

    let map = load(root).unwrap();
    assert_eq!(map.packages.len(), 1);
    assert_eq!(map.packages[0].name, "@x/api");
}

#[test]
fn pnpm_workspace_yaml_takes_precedence_over_package_json_workspaces() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    write(
        &root.join("package.json"),
        r#"{"workspaces": ["npm-packages/*"]}"#,
    );
    write(
        &root.join("pnpm-workspace.yaml"),
        "packages:\n  - pnpm-packages/*\n",
    );
    write(
        &root.join("npm-packages/api/package.json"),
        r#"{"name": "@x/npm"}"#,
    );
    write(
        &root.join("pnpm-packages/api/package.json"),
        r#"{"name": "@x/pnpm"}"#,
    );

    let map = load(root).unwrap();
    assert_eq!(map.packages.len(), 1);
    assert_eq!(map.packages[0].name, "@x/pnpm");
}

#[test]
fn pnpm_workspace_without_packages_includes_direct_subdirectories() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    write(&root.join("pnpm-workspace.yaml"), "{}\n");
    write(&root.join("api/package.json"), r#"{"name": "@x/api"}"#);
    write(
        &root.join("api/fixtures/nested/package.json"),
        r#"{"name": "@x/nested"}"#,
    );

    let map = load(root).unwrap();
    assert_eq!(map.packages.len(), 1);
    assert_eq!(map.packages[0].name, "@x/api");
}

#[test]
fn pnpm_workspace_exclusion_globs_remove_loaded_packages() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    write(
        &root.join("pnpm-workspace.yaml"),
        "packages:\n  - packages/**\n  - '!packages/**/fixtures/**'\n",
    );
    write(
        &root.join("packages/group/foo/package.json"),
        r#"{"name": "@x/foo"}"#,
    );
    write(
        &root.join("packages/group/foo/fixtures/bar/package.json"),
        r#"{"name": "@x/bar"}"#,
    );

    let map = load(root).unwrap();
    assert_eq!(map.packages.len(), 1);
    assert_eq!(map.packages[0].name, "@x/foo");
}

// ── Workspace package entries ─────────────────────────────────────────

#[test]
fn resolve_package_finds_by_name() {
    let dir = TempDir::new().unwrap();
    let entry = dir.path().join("src/index.mts");
    write(&entry, "");
    let map = WorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@x/api".to_string(),
        dir: dir.path().to_path_buf(),
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
