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
    assert!(!scan::test_support::package_json_has_next_dependency(
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
    let remapper =
        crate::codebase::ts_source::FrozenPathRemapper::from_paths(files.keys().cloned());
    let mut visiting = HashSet::new();
    let mut memo = HashMap::new();
    assert!(collector::collect_file_exports(
        &root.join("src/not-present.ts"),
        &files,
        &resolver,
        &workspace,
        &remapper,
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
            &remapper,
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
            &remapper,
            &mut visiting,
        ),
        None
    );
}

#[test]
fn deferred_reexports_keep_named_reexports_lexically_visible() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/unique-exports-suppressed-origin"),
    );
    let all_files = discover_files(&root, &[]);
    let source_files = scan::test_support::collect_source_files(&root, &all_files).unwrap();
    let files: HashMap<PathBuf, SourceFile> = source_files
        .into_iter()
        .map(|mut file| {
            // Aggregate checking defers directive filtering until after origin
            // canonicalization, so every fixture file must follow that path.
            file.defer_suppression = true;
            if file.disabled {
                // The standalone fixture helper intentionally omits symbols
                // for disabled files; deferred checking still needs those
                // exports to carry their suppression provenance.
                file.symbols = crate::codebase::ts_symbols::extract_symbols_at_path(
                    &file.path,
                    &file.source,
                    false,
                )
                .unwrap()
                .into();
            }
            (file.path.clone(), file)
        })
        .collect();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig);
    let workspace = WorkspaceMap::default();
    let remapper =
        crate::codebase::ts_source::FrozenPathRemapper::from_paths(files.keys().cloned());
    assert_eq!(
        super::super::origin::resolve_export_source(
            "./source",
            &root.join("src/barrel.ts"),
            &resolver,
            &workspace,
            &remapper,
        ),
        Some(root.join("src/source.ts"))
    );
    let collect = |relative: &str| {
        let mut visiting = HashSet::new();
        let mut memo = HashMap::new();
        collector::collect_file_exports(
            &root.join(relative),
            &files,
            &resolver,
            &workspace,
            &remapper,
            &mut visiting,
            &mut memo,
        )
    };

    let explicit = collect("src/barrel.ts");
    assert_eq!(explicit.len(), 1);
    let explicit_origin = &explicit[0].origin;
    // A disabled target has no exported identity for an unsuppressed named
    // re-export to inherit. The barrel occurrence must therefore remain
    // visible and use its own lexical origin.
    assert!(!explicit[0].suppressed, "{explicit:#?}");
    assert_eq!(explicit[0].suppression_location, None);
    assert_eq!(explicit_origin.file, "src/barrel.ts");
    assert_eq!(explicit_origin.line, 2);
    assert_eq!(explicit_origin.name, "Shared");
    assert_eq!(explicit_origin.bucket, ExportBucket::Value);
    assert!(!explicit_origin.suppressed);
    assert_eq!(explicit_origin.suppression_location, None);

    let wildcard = collect("src/wild-barrel.ts");
    assert_eq!(wildcard.len(), 2);
    assert!(wildcard.iter().all(|occurrence| occurrence.suppressed));
    assert!(wildcard.iter().all(|occurrence| {
        occurrence.suppression_location.as_ref() == Some(&("src/wild-barrel.ts".to_string(), 4))
    }));

    let suppressed_barrel = collect("src/suppressed-barrel.ts");
    assert_eq!(suppressed_barrel.len(), 1);
    assert!(suppressed_barrel[0].suppressed);
    assert_eq!(
        suppressed_barrel[0].suppression_location.as_ref(),
        Some(&("src/suppressed-barrel.ts".to_string(), 3))
    );
    let mut suppressed_origin_visiting = HashSet::new();
    let suppressed_origin = collector::find_target_export_origin(
        &root.join("src/suppressed-barrel.ts"),
        "Shared",
        &files,
        &resolver,
        &workspace,
        &remapper,
        &mut suppressed_origin_visiting,
    )
    .unwrap();
    assert_eq!(
        suppressed_origin.suppression_location.as_ref(),
        Some(&("src/suppressed-barrel.ts".to_string(), 3))
    );

    let missing = root.join("src/missing.ts");
    let missing_remapper =
        crate::codebase::ts_source::FrozenPathRemapper::from_paths([missing.clone()]);
    let mut visiting = HashSet::new();
    assert_eq!(
        collector::find_target_export_origin(
            &missing,
            "Missing",
            &files,
            &resolver,
            &workspace,
            &missing_remapper,
            &mut visiting,
        ),
        None
    );

    let mut equivalent = explicit_origin.clone();
    equivalent.suppressed = !equivalent.suppressed;
    equivalent.suppression_location = None;
    assert_eq!(*explicit_origin, equivalent);
    assert_eq!(
        explicit_origin.partial_cmp(&equivalent),
        Some(std::cmp::Ordering::Equal)
    );

    assert!(!scan::test_support::package_json_has_next_dependency(
        &root.join("package.json")
    ));
    assert!(!scan::test_support::package_json_has_next_dependency(
        &root.join("missing-package.json")
    ));
}
