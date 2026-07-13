use super::*;

#[test]
fn scan_helpers_cover_filter_and_parse_edges() {
    let root = fixture("unique-exports-edge-cases");
    let files = vec![root.join("src/direct.ts"), root.join("package.json")];
    let filtered = scan::filter_source_files(&files);
    assert_eq!(filtered.len(), 1);
    let sources = scan::test_support::collect_source_files(&root, &filtered).unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].rel, "src/direct.ts");
    assert!(
        scan::test_support::collect_source_files(&root, &[root.join("src/not-present.ts")])
            .is_err()
    );
    let invalid_root = fixture("unique-exports-invalid-source");
    let error = scan::test_support::collect_source_files(
        &invalid_root,
        &[invalid_root.join("src/broken.ts")],
    )
    .unwrap_err();
    assert!(format!("{error:#}").contains("extracting symbols from"));
    let disabled_invalid =
        scan::test_support::collect_source_files(&root, &[root.join("src/disabled-invalid.ts")])
            .unwrap();
    assert!(disabled_invalid[0].disabled);
    assert!(disabled_invalid[0].symbols.exports.is_empty());
    let next_root = fixture("unique-exports-nextjs");
    let next_visible = discover_files(&next_root, &[]);
    let lookup = scan::NextJsProjectLookup::new(&next_root, &[], &next_visible);
    assert!(!lookup.contains_file(&root.join("src/direct.ts")));
    let lookup = scan::NextJsProjectLookup::new(&root, &[PathBuf::from("loose.ts")], &[]);
    assert!(!lookup.contains_file(Path::new("loose.ts")));
    // PathBuf::from("/") has parent() == None, exercising the unwrap_or_else fallback.
    let lookup = scan::NextJsProjectLookup::new(&root, &[PathBuf::from("/")], &[]);
    assert!(!lookup.contains_file(Path::new("/")));
    assert!(!scan::package_json_has_next_dependency(
        &fixture("unique-exports-malformed-package").join("package.json")
    ));
}

#[test]
fn defensive_helpers_ignore_missing_targets_and_non_matching_default_exports() {
    let root = fixture("unique-exports-edge-cases");
    let all_files = discover_files(&root, &[]);
    let mut files = scan::filter_source_files(&all_files);
    files.retain(|file| file.file_name().and_then(|name| name.to_str()) != Some("invalid.ts"));
    let source_files = scan::test_support::collect_source_files(&root, &files).unwrap();
    let files: HashMap<PathBuf, SourceFile> = source_files
        .into_iter()
        .map(|file| (file.path.clone(), file))
        .collect();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig);
    let workspace = WorkspaceMap::default();
    let mut visiting = HashSet::new();
    let mut memo = HashMap::new();
    assert!(collector::collect_file_exports(
        &root.join("src/not-present.ts"),
        &files,
        &resolver,
        &workspace,
        &mut visiting,
        &mut memo,
    )
    .is_empty());
    let mut visiting = HashSet::new();
    assert_eq!(
        collector::find_target_export_origin(
            &root.join("src/not-present.ts"),
            "Missing",
            &files,
            &resolver,
            &workspace,
            &mut visiting,
        ),
        None
    );
    let mut visiting = HashSet::new();
    assert_eq!(
        collector::find_target_export_origin(
            &root.join("src/default-source.ts"),
            "NotDefault",
            &files,
            &resolver,
            &workspace,
            &mut visiting,
        ),
        None
    );
}
