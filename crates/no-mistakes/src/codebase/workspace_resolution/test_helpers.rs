use super::*;

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

#[test]
fn export_pattern_sort_uses_pattern_name_as_tiebreaker() {
    let b_pattern = "./b/*".to_string();
    let a_pattern = "./a/*".to_string();
    let b_value = serde_json::json!("./b/*.mts");
    let a_value = serde_json::json!("./a/*.mts");
    let mut patterns = [(&b_pattern, &b_value, 4), (&a_pattern, &a_value, 4)];

    patterns.sort_by(compare_export_patterns);

    assert_eq!(patterns[0].0, "./a/*");
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
fn resolve_specifier_rejects_unprefixed_subpath_without_exports() {
    let fixture = crate::test_support::fixture_root("workspaces").join("package-without-exports");
    let dir = crate::test_support::materialize_saved_fixture(&fixture);
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
