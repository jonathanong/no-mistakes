use super::*;
use clap::Parser;

mod extra;

fn parse(argv: &[&str]) -> TraverseArgs {
    TraverseArgs::parse_from(argv)
}

fn build_graph(root: &Path, tsconfig: &crate::codebase::ts_resolver::TsConfig) -> graph::DepGraph {
    let graph_files = graph::GraphFiles::discover(root);
    graph::DepGraph::build_with_plan_and_files(
        root,
        tsconfig,
        graph::GraphBuildPlan::all(),
        &graph_files,
    )
}

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase-analysis")
            .join(name),
    )
}

#[test]
fn run_surfaces_tsconfig_errors() {
    let root = fixture_root("symbols-output");
    let args = TraverseArgs {
        files: vec![PathBuf::from("src/utils.mts")],
        root: Some(root.clone()),
        tsconfig: Some(root.join("tsconfig-invalid.json")),
        depth: None,
        filters: Vec::new(),
        tests: Vec::new(),
        format: Some(Format::Json),
        json: false,
        relationships: Vec::new(),
        timings: false,
    };

    let err = run(args, Direction::Deps).unwrap_err();

    assert!(format!("{err:#}").contains("tsconfig-invalid.json"));
}

// ── TraverseArgs parsing ────────────────────────────────────────────────

#[test]
fn files_parsed() {
    let a = parse(&["deps", "src/main.mts"]);
    assert_eq!(a.files, vec![PathBuf::from("src/main.mts")]);
    assert!(a.depth.is_none());
    assert!(a.filters.is_empty());
}

#[test]
fn depth_flag_parsed() {
    let a = parse(&["deps", "a.mts", "--depth", "3"]);
    assert_eq!(a.depth, Some(3));
}

#[test]
fn filter_flag_parsed() {
    let a = parse(&["deps", "a.mts", "--filter", "**/*.test.mts"]);
    assert_eq!(a.filters, vec!["**/*.test.mts"]);
}

#[test]
fn filter_flag_repeatable() {
    let a = parse(&[
        "deps",
        "a.mts",
        "--filter",
        "**/*.test.mts",
        "--filter",
        "**/*.spec.mts",
    ]);
    assert_eq!(a.filters.len(), 2);
}

#[test]
fn root_flag_parsed() {
    let a = parse(&["deps", "a.mts", "--root", "/some/path"]);
    assert_eq!(a.root, Some(PathBuf::from("/some/path")));
}

#[test]
fn multiple_input_files_parsed() {
    let a = parse(&["deps", "a.mts", "b.mts", "c.mts"]);
    assert_eq!(a.files.len(), 3);
}

#[test]
fn format_flag_parsed() {
    let a = parse(&["deps", "a.mts", "--format", "md"]);
    assert_eq!(a.format, Some(Format::Md));
}

#[test]
fn format_json_variant() {
    let a = parse(&["deps", "a.mts", "--format", "json"]);
    assert_eq!(a.format, Some(Format::Json));
}

#[test]
fn format_yml_variant() {
    let a = parse(&["deps", "a.mts", "--format", "yml"]);
    assert_eq!(a.format, Some(Format::Yml));
}

#[test]
fn format_paths_variant() {
    let a = parse(&["deps", "a.mts", "--format", "paths"]);
    assert_eq!(a.format, Some(Format::Paths));
}

#[test]
fn format_human_variant() {
    let a = parse(&["deps", "a.mts", "--format", "human"]);
    assert_eq!(a.format, Some(Format::Human));
}

#[test]
fn json_flag_conflicts_with_format() {
    let result = TraverseArgs::try_parse_from(["deps", "a.mts", "--json", "--format", "human"]);
    assert!(result.is_err());
}

#[test]
fn test_flag_parsed() {
    let a = parse(&["deps", "a.mts", "--test", "vitest"]);
    assert_eq!(a.tests, vec!["vitest"]);
}

#[test]
fn test_flag_repeatable() {
    let a = parse(&["deps", "a.mts", "--test", "vitest", "--test", "playwright"]);
    assert_eq!(a.tests.len(), 2);
}

// ── test_globs expansion ────────────────────────────────────────────────

#[test]
fn vitest_globs_include_test_mts() {
    let globs = test_globs("vitest");
    assert!(globs.iter().any(|g| g == "**/*.test.mts"));
    assert!(globs.iter().any(|g| g == "**/*.spec.ts"));
}

#[test]
fn playwright_globs_include_e2e() {
    let globs = test_globs("playwright");
    assert!(globs.contains(&"**/tests/e2e/**/*.mts".to_string()));
    assert!(globs.contains(&"**/playwright/**/*.spec.mts".to_string()));
    assert!(globs.contains(&"**/playwright/**/*.spec.js".to_string()));
}

#[test]
fn cargo_globs_include_tests_dir() {
    let globs = test_globs("cargo");
    assert!(globs.iter().any(|g| g.contains("tests/**/*.rs")));
}

#[test]
fn unknown_framework_returns_empty() {
    let globs = test_globs("unknown");
    assert!(globs.is_empty());
}

#[test]
fn jest_globs_match_vitest_style_test_files() {
    let globs = test_globs("jest");
    assert!(globs.iter().any(|g| g == "**/*.test.mts"));
    assert!(globs.iter().any(|g| g == "**/*.spec.ts"));
}

// ── --relationship / relationship_filter ─────────────────────────────────

#[test]
fn relationship_flag_parsed() {
    let a = parse(&["deps", "a.mts", "--relationship", "import"]);
    assert_eq!(a.relationships, vec![RelationshipArg::Import]);
}

#[test]
fn relationship_flag_repeatable() {
    let a = parse(&[
        "deps",
        "a.mts",
        "--relationship",
        "import",
        "--relationship",
        "test",
    ]);
    assert_eq!(a.relationships.len(), 2);
}

#[test]
fn empty_relationships_returns_none() {
    assert!(relationship_filter(&[]).is_none());
}

#[test]
fn all_keyword_returns_none() {
    assert!(relationship_filter(&[RelationshipArg::All]).is_none());
}

#[test]
fn import_maps_to_all_import_forms() {
    let set = relationship_filter(&[RelationshipArg::Import]).unwrap();
    assert!(set.contains(&EdgeKind::Import));
    assert!(set.contains(&EdgeKind::TypeImport));
    assert!(set.contains(&EdgeKind::DynamicImport));
    assert!(set.contains(&EdgeKind::Require));
    assert!(!set.contains(&EdgeKind::TestOf));
}

#[test]
fn granular_imports_map_to_respective_edge_kinds() {
    let static_set = relationship_filter(&[RelationshipArg::ImportStatic]).unwrap();
    assert!(static_set.contains(&EdgeKind::Import));
    assert!(!static_set.contains(&EdgeKind::TypeImport));
    assert!(!static_set.contains(&EdgeKind::DynamicImport));
    assert!(!static_set.contains(&EdgeKind::Require));

    let dynamic_set = relationship_filter(&[RelationshipArg::ImportDynamic]).unwrap();
    assert!(!dynamic_set.contains(&EdgeKind::Import));
    assert!(!dynamic_set.contains(&EdgeKind::TypeImport));
    assert!(dynamic_set.contains(&EdgeKind::DynamicImport));
    assert!(!dynamic_set.contains(&EdgeKind::Require));

    let type_set = relationship_filter(&[RelationshipArg::ImportType]).unwrap();
    assert!(!type_set.contains(&EdgeKind::Import));
    assert!(type_set.contains(&EdgeKind::TypeImport));
    assert!(!type_set.contains(&EdgeKind::DynamicImport));
    assert!(!type_set.contains(&EdgeKind::Require));

    let require_set = relationship_filter(&[RelationshipArg::ImportRequire]).unwrap();
    assert!(!require_set.contains(&EdgeKind::Import));
    assert!(!require_set.contains(&EdgeKind::TypeImport));
    assert!(!require_set.contains(&EdgeKind::DynamicImport));
    assert!(require_set.contains(&EdgeKind::Require));
}

#[test]
fn granular_import_cli_flags_parsed() {
    let a = parse(&[
        "deps",
        "a.mts",
        "--relationship",
        "import-static",
        "--relationship",
        "import-dynamic",
        "--relationship",
        "import-type",
        "--relationship",
        "import-require",
    ]);
    assert_eq!(
        a.relationships,
        vec![
            RelationshipArg::ImportStatic,
            RelationshipArg::ImportDynamic,
            RelationshipArg::ImportType,
            RelationshipArg::ImportRequire,
        ]
    );
}

#[test]
fn workspace_maps_to_workspace_import() {
    let set = relationship_filter(&[RelationshipArg::Workspace]).unwrap();
    assert!(set.contains(&EdgeKind::WorkspaceImport));
}

#[test]
fn test_maps_to_test_of_and_route_test() {
    let set = relationship_filter(&[RelationshipArg::Test]).unwrap();
    assert!(set.contains(&EdgeKind::TestOf));
    assert!(set.contains(&EdgeKind::RouteTest));
    assert!(set.contains(&EdgeKind::Layout));
}

#[test]
fn route_maps_to_route_ref_and_route_test() {
    let set = relationship_filter(&[RelationshipArg::Route]).unwrap();
    assert!(set.contains(&EdgeKind::RouteRef));
    assert!(set.contains(&EdgeKind::RouteTest));
    assert!(set.contains(&EdgeKind::Layout));
}

#[test]
fn queue_maps_to_queue_enqueue_and_queue_worker() {
    let set = relationship_filter(&[RelationshipArg::Queue]).unwrap();
    assert!(set.contains(&EdgeKind::QueueEnqueue));
    assert!(set.contains(&EdgeKind::QueueWorker));
}

#[test]
fn md_maps_to_markdown_link() {
    let set = relationship_filter(&[RelationshipArg::Md]).unwrap();
    assert!(set.contains(&EdgeKind::MarkdownLink));
}

#[test]
fn ci_maps_to_ci_invocation() {
    let set = relationship_filter(&[RelationshipArg::Ci]).unwrap();
    assert!(set.contains(&EdgeKind::CiInvocation));
}

#[test]
fn multiple_kinds_combined() {
    let set = relationship_filter(&[RelationshipArg::Import, RelationshipArg::Test]).unwrap();
    assert!(set.contains(&EdgeKind::Import));
    assert!(set.contains(&EdgeKind::TestOf));
    assert!(!set.contains(&EdgeKind::QueueEnqueue));
    assert!(!set.contains(&EdgeKind::QueueWorker));
}

// ── parse_entrypoint ────────────────────────────────────────────────────

#[test]
fn parse_plain_path() {
    let ep = parse_entrypoint("src/main.mts");
    assert_eq!(ep.file, PathBuf::from("src/main.mts"));
    assert!(ep.symbol.is_none());
}

#[test]
fn parse_path_with_symbol() {
    let ep = parse_entrypoint("src/queues.mts#enqueueBulkTopicEmbeddings");
    assert_eq!(ep.file, PathBuf::from("src/queues.mts"));
    assert_eq!(ep.symbol.as_deref(), Some("enqueueBulkTopicEmbeddings"));
}

#[test]
fn parse_path_multiple_hashes_splits_on_first() {
    let ep = parse_entrypoint("src/foo.mts#sym#extra");
    assert_eq!(ep.symbol.as_deref(), Some("sym#extra"));
}

#[test]
fn resolve_root_uses_absolute_path() {
    let root = fixture_root("simple");
    let args = {
        let args = vec![
            "deps".to_string(),
            "--root".to_string(),
            root.to_string_lossy().into_owned(),
            "a.mts".to_string(),
        ];
        TraverseArgs::parse_from(args)
    };
    let cwd = fixture_root("filter");
    let resolved_root = resolve_root(&args, &cwd);
    assert_eq!(resolved_root, root);
}

#[test]
fn resolve_root_joins_relative_with_cwd() {
    let args = parse(&["deps", "--root", "sub/dir", "a.mts"]);
    let cwd = fixture_root("filter");
    let root = resolve_root(&args, &cwd);
    assert_eq!(root, cwd.join("sub/dir"));
}

#[test]
fn resolve_entrypoints_prefers_root_before_cwd_fallback() {
    let root = fixture_root("simple");
    let args = parse(&[
        "deps",
        "a.mts",
        "does-not-exist.mts",
        "../../other.mts#exportName",
    ]);
    let cwd = fixture_root("simple").join("src");
    let entrypoints = resolve_entrypoints(&args.files, &root, &cwd);

    assert_eq!(entrypoints[0].file, root.join("a.mts"));
    assert_eq!(entrypoints[0].symbol, None);
    assert_eq!(entrypoints[1].file, cwd.join("does-not-exist.mts"));
    assert_eq!(entrypoints[1].symbol, None);
    assert_eq!(entrypoints[2].file, cwd.join("../../other.mts"));
    assert_eq!(entrypoints[2].symbol.as_deref(), Some("exportName"));
}

#[test]
fn validate_direction_allows_symbol_with_dependents() {
    let args = parse(&["deps", "a.mts#alpha", "b.mts"]);
    let root = fixture_root("simple");
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);
    validate_direction(&Direction::Dependents, &entrypoints).unwrap();
}

#[test]
fn validate_direction_rejects_symbol_with_deps() {
    let args = parse(&["deps", "a.mts#alpha"]);
    let root = fixture_root("simple");
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);
    let err = validate_direction(&Direction::Deps, &entrypoints).unwrap_err();
    assert!(format!("{err}").contains("#symbol"));
}
