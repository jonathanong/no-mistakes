#[test]
fn symbol_index_basic_lookup() {
    let mut map: HashMap<PathBuf, Vec<(PathBuf, String, String, bool)>> = HashMap::new();
    map.insert(
        p("/src/b.mts"),
        vec![(
            p("/src/a.mts"),
            "alpha".to_string(),
            "alpha".to_string(),
            false,
        )],
    );
    let index = SymbolIndex::build(&map);
    let importers = index
        .importers_of(p("/src/a.mts").as_path(), "alpha")
        .unwrap();
    assert_eq!(importers.len(), 1);
    assert_eq!(importers[0].0.as_ref(), p("/src/b.mts").as_path());
}

#[test]
fn symbol_index_missing_returns_none() {
    let map: HashMap<PathBuf, Vec<(PathBuf, String, String, bool)>> = HashMap::new();
    let index = SymbolIndex::build(&map);
    assert!(index
        .importers_of(p("/src/a.mts").as_path(), "ghost")
        .is_none());
}

#[test]
fn symbol_index_caches_sorted_unique_file_importers_across_symbols_and_reexports() {
    let source = p("/src/source.mts");
    let importer_a = p("/src/a.mts");
    let importer_b = p("/src/b.mts");
    let map = HashMap::from([
        (
            importer_b.clone(),
            vec![
                (
                    source.clone(),
                    "beta".to_string(),
                    "beta".to_string(),
                    false,
                ),
                (
                    source.clone(),
                    "gamma".to_string(),
                    "gamma".to_string(),
                    true,
                ),
            ],
        ),
        (
            importer_a.clone(),
            vec![
                (
                    source.clone(),
                    "alpha".to_string(),
                    "alpha".to_string(),
                    false,
                ),
                (
                    source.clone(),
                    "beta".to_string(),
                    "aliasedBeta".to_string(),
                    false,
                ),
            ],
        ),
    ]);

    let index = SymbolIndex::build(&map);

    assert_eq!(index.file_importers(&source), vec![importer_a, importer_b]);
    assert_eq!(index.importers_of(&source, "beta").unwrap().len(), 2);
}

fn interned_path_hash(path: &std::sync::Arc<std::path::Path>) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn symbol_index_interned_paths_from_distinct_pathbuf_allocations_sort_and_hash_equal() {
    use std::cmp::Ordering;
    use std::sync::Arc;

    let cases = [
        "/src/a.mts",
        "src/widget.ts",
        "/tmp/foo/bar.ts",
        "C:\\repo\\app\\index.ts",
    ];
    for path in cases {
        let left = PathBuf::from(path);
        let right = PathBuf::from(String::from(path));
        assert_ne!(
            left.as_os_str().as_encoded_bytes().as_ptr(),
            right.as_os_str().as_encoded_bytes().as_ptr(),
            "table case {path} should use distinct PathBuf allocations"
        );

        let interned_left = intern_symbol_index_path(&left);
        let interned_right = intern_symbol_index_path(&right);
        assert!(!Arc::ptr_eq(&interned_left, &interned_right));
        assert_eq!(interned_left, interned_right);
        assert_eq!(
            interned_path_hash(&interned_left),
            interned_path_hash(&interned_right)
        );
        assert_eq!(
            interned_left.as_os_str().cmp(interned_right.as_os_str()),
            Ordering::Equal
        );

        let source = PathBuf::from("/src/source.mts");
        let lookup = PathBuf::from(String::from("/src/source.mts"));
        let map = HashMap::from([(
            left.clone(),
            vec![(
                source.clone(),
                "alpha".to_string(),
                "alpha".to_string(),
                false,
            )],
        )]);
        let index = SymbolIndex::build(&map);
        assert!(index.importers_of(&lookup, "alpha").is_some());
        assert_eq!(index.file_importers(&lookup), vec![right]);
    }
}

#[test]
fn symbol_index_wide_fanout_query_preserves_each_distinct_source() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/queries/symbol-index-wide-fanout"),
    );
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let index = SymbolIndex::build_from_root(&root, &tsconfig).unwrap();
    let importer = root.join("src/importer.mts");

    // A wide fanout must retain one reverse-index row per source without
    // changing the importer ordering or symbol-level query results.
    for (source, symbol) in [
        ("source-a.mts", "alpha"),
        ("source-b.mts", "beta"),
        ("source-c.mts", "gamma"),
        ("source-d.mts", "delta"),
        ("source-e.mts", "epsilon"),
        ("source-f.mts", "zeta"),
    ] {
        let source = root.join("src").join(source);
        let importers = index.importers_of(&source, symbol).unwrap();
        assert_eq!(importers.len(), 1);
        assert_eq!(importers[0].0.as_ref(), importer.as_path());
        assert_eq!(index.file_importers(&source), vec![importer.clone()]);
    }
}

#[test]
fn symbol_index_bucket_initial_capacity_is_bounded_for_wide_fanout() {
    assert_eq!(source_bucket_initial_capacity(0), 0);
    assert_eq!(source_bucket_initial_capacity(8), 8);
    assert_eq!(source_bucket_initial_capacity(2_048), MAX_SOURCE_BUCKET_INITIAL_CAPACITY);
}
