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
    assert_eq!(importers[0].0, p("/src/b.mts"));
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
