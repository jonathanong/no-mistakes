#[test]
fn resolve_entrypoints_keeps_directory_without_entry_as_file_node() {
    let root = fixture_root("graph-empty-dir");
    let args = parse(&["deps", "empty"]);
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);

    assert_eq!(entrypoints[0].node, graph::NodeId::file(root.join("empty")));
    assert_eq!(entrypoints[0].file, root.join("empty"));
}

#[test]
fn resolve_entrypoints_accepts_workspace_package_specifier() {
    let root = fixture_root("graph-modules");
    let args = parse(&["deps", "@local/pkg"]);
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);

    assert_eq!(
        entrypoints[0].node,
        graph::NodeId::file(root.join("packages/local/src/index.mts"))
    );
    assert_eq!(
        entrypoints[0].file,
        root.join("packages/local/src/index.mts")
    );
}

#[test]
fn resolve_entrypoints_strips_symbol_suffix_from_module_node() {
    let root = fixture_root("graph-modules");
    let args = parse(&["dependents", "@external/pkg#handler"]);
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);

    assert_eq!(entrypoints[0].node, graph::NodeId::module("@external/pkg"));
    assert_eq!(entrypoints[0].symbol.as_deref(), Some("handler"));
}

#[test]
fn resolve_entrypoints_keeps_package_subpath_with_extension_as_module_node() {
    let root = fixture_root("graph-modules");
    let args = parse(&["dependents", "lodash", "lodash/fp.js"]);
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);

    assert_eq!(entrypoints[0].node, graph::NodeId::module("lodash"));
    assert_eq!(entrypoints[1].node, graph::NodeId::module("lodash/fp.js"));
}

#[test]
fn resolve_entrypoints_treats_missing_source_path_with_existing_parent_as_file_node() {
    let root = fixture_root("graph-modules");
    let args = parse(&["dependents", "src/new-file.ts"]);
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);

    assert_eq!(
        entrypoints[0].node,
        graph::NodeId::file(root.join("src/new-file.ts"))
    );
}

#[test]
fn explicit_directory_does_not_infer_a_gitignored_index_file() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass3-visibility");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let ignored_index = fixture.path().join("explicit-dir/index.ts");
    assert!(ignored_index.exists());

    let entrypoints = resolve_entrypoints(
        &[PathBuf::from("explicit-dir")],
        fixture.path(),
        fixture.path(),
    );

    assert_eq!(
        entrypoints[0].node,
        graph::NodeId::file(fixture.path().join("explicit-dir"))
    );
    assert_ne!(entrypoints[0].file, ignored_index);
}

#[test]
fn entrypoint_package_helpers_cover_relative_scoped_and_invalid_roots() {
    let modules_root = fixture_root("graph-modules");
    let module_files = graph::GraphFiles::discover(&modules_root);
    let module_dependencies = root_dependency_names(&modules_root, module_files.all());
    assert_eq!(raw_package_name("./local/file.ts"), None);
    assert_eq!(
        raw_package_name("@scope/pkg/subpath.js").as_deref(),
        Some("@scope/pkg")
    );
    assert!(!raw_looks_like_source_file(
        "lodash/fp.js",
        &modules_root.join("lodash/fp.js"),
        &module_dependencies
    ));

    let simple_root = fixture_root("simple");
    let simple_files = graph::GraphFiles::discover(&simple_root);
    assert!(!root_dependency_names(&simple_root, simple_files.all()).contains("lodash"));
    let malformed_root = fixture_root("unique-exports-malformed-package");
    let malformed_files = graph::GraphFiles::discover(&malformed_root);
    assert!(!root_dependency_names(&malformed_root, malformed_files.all()).contains("lodash"));
}

#[test]
fn validate_direction_allows_symbol_with_dependents() {
    let args = parse(&["deps", "a.mts#alpha", "b.mts"]);
    let root = fixture_root("simple");
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);
    validate_direction(&Direction::Dependents, &entrypoints, false).unwrap();
}

#[test]
fn validate_direction_rejects_symbol_with_deps() {
    let args = parse(&["deps", "a.mts#alpha"]);
    let root = fixture_root("simple");
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);
    let err = validate_direction(&Direction::Deps, &entrypoints, false).unwrap_err();
    assert!(format!("{err}").contains("#symbol"));
}
