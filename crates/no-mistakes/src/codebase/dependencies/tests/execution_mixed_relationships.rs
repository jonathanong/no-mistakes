#[test]
fn cli_mixed_import_and_call_relationships_traverse_as_one_union() {
    let root = fixture_root("mixed-relationship-traversal");
    let output = run_json(
        TraverseArgs {
            files: vec![PathBuf::from("src/entry.mts")],
            file_symbols: Vec::new(),
            file_entrypoints_are_structured: Vec::new(),
            root: Some(root),
            tsconfig: None,
            depth: None,
            filters: Vec::new(),
            target_modules: Vec::new(),
            tests: Vec::new(),
            format: Some(Format::Json),
            json: true,
            relationships: vec![RelationshipArg::Import, RelationshipArg::Call],
            include_symbols: false,
            timings: false,
        },
        Direction::Deps,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let paths = value["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| {
            entry
                .get("file")
                .or_else(|| entry.get("path"))
                .and_then(serde_json::Value::as_str)
        })
        .collect::<Vec<_>>();

    assert!(
        paths.iter().any(|path| path.ends_with("src/imported.mts")),
        "{value}"
    );
    assert!(
        paths.iter().any(|path| path.ends_with("src/called.mts")),
        "{value}"
    );
}

#[test]
fn cli_mixed_import_and_call_dependents_traverse_as_one_union() {
    let root = fixture_root("mixed-relationship-traversal");
    let output = run_json(
        TraverseArgs {
            files: vec![PathBuf::from("src/called.mts")],
            file_symbols: vec![Some("called".to_string())],
            file_entrypoints_are_structured: Vec::new(),
            root: Some(root),
            tsconfig: None,
            depth: None,
            filters: Vec::new(),
            target_modules: Vec::new(),
            tests: Vec::new(),
            format: Some(Format::Json),
            json: true,
            relationships: vec![RelationshipArg::Import, RelationshipArg::Call],
            include_symbols: false,
            timings: false,
        },
        Direction::Dependents,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let paths = value["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| {
            entry
                .get("file")
                .or_else(|| entry.get("path"))
                .and_then(serde_json::Value::as_str)
        })
        .collect::<Vec<_>>();

    assert!(
        paths.iter().any(|path| path.ends_with("src/imported.mts")),
        "{value}"
    );
    assert!(
        paths.iter().any(|path| path.ends_with("src/entry.mts")),
        "{value}"
    );
}

#[test]
fn cli_call_symbol_query_omits_unrelated_top_level_calls() {
    let root = fixture_root("call-traversal");
    let output = run_json(
        TraverseArgs {
            files: vec![PathBuf::from("src/selected-call.mts#selected")],
            file_symbols: Vec::new(),
            file_entrypoints_are_structured: Vec::new(),
            root: Some(root),
            tsconfig: None,
            depth: None,
            filters: Vec::new(),
            target_modules: Vec::new(),
            tests: Vec::new(),
            format: Some(Format::Json),
            json: true,
            relationships: vec![RelationshipArg::Call],
            include_symbols: false,
            timings: false,
        },
        Direction::Deps,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let paths = value["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| {
            entry
                .get("file")
                .or_else(|| entry.get("path"))
                .and_then(serde_json::Value::as_str)
        })
        .collect::<Vec<_>>();

    assert!(
        paths
            .iter()
            .any(|path| path.ends_with("src/selected-call-target.mts")),
        "{value}"
    );
    assert!(
        !paths
            .iter()
            .any(|path| path.ends_with("src/selected-call-unrelated.mts")),
        "{value}"
    );
}

#[test]
fn validate_direction_allows_deps_symbols_for_call_queries() {
    let args = parse(&["deps", "a.mts#alpha"]);
    let root = fixture_root("simple");
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);
    validate_direction(&Direction::Deps, &entrypoints, true).unwrap();
}
