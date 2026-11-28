use super::*;

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
