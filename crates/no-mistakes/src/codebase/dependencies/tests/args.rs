use super::root_dependency_test_helpers::root_dependency_names;
use super::traversal_entrypoint_test_helpers::resolve_entrypoints_with_files;
use super::*;
use super::traversal::*;
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
    .expect("test graph builds")
}

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn resolve_entrypoints(raw_entrypoints: &[PathBuf], root: &Path, cwd: &Path) -> Vec<Entrypoint> {
    let graph_files = graph::GraphFiles::discover(root);
    resolve_entrypoints_with_files(
        raw_entrypoints,
        &[],
        &[],
        root,
        cwd,
        &graph_files,
        false,
    )
}

#[test]
fn run_surfaces_tsconfig_errors() {
    let root = fixture_root("symbols-output");
    let args = TraverseArgs {
        files: vec![PathBuf::from("src/utils.mts")],
        file_symbols: Vec::new(),
        file_entrypoints_are_structured: Vec::new(),
        root: Some(root.clone()),
        tsconfig: Some(root.join("tsconfig-invalid.json")),
        depth: None,
        filters: Vec::new(),
        target_modules: Vec::new(),
        tests: Vec::new(),
        format: Some(Format::Json),
        json: false,
        relationships: Vec::new(),
        include_symbols: false,
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
fn target_module_flag_repeatable() {
    let a = parse(&[
        "deps",
        "a.mts",
        "--target-module",
        "@react/*",
        "--target-module",
        "lodash",
    ]);
    assert_eq!(a.target_modules, vec!["@react/*", "lodash"]);
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
fn project_discovery_test_filters_escape_literal_paths_and_fallback_when_empty() {
    let root = fixture_root("test-plan-project-discovery");
    let globs = prepared_test_filters(&root, "playwright");
    assert!(globs.contains(&"e2e/\\[locale\\].pw.ts".to_string()));

    let fallback = prepared_test_filters(Path::new("/repo"), "vitest");
    assert!(fallback.contains(&"**/*.test.ts".to_string()));
}

fn prepared_test_filters(root: &Path, framework: &str) -> Vec<String> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible = snapshot.paths_for(root);
    let config = crate::config::v2::load_v2_config_from_visible(root, None, &visible)
        .unwrap_or_default();
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        None,
        root,
        &visible,
    )
    .unwrap();
    test_filters_from_prepared(root, framework, &config, &tsconfig, &snapshot, None)
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

include!("args_relationships.rs");

// ── parse_entrypoint ────────────────────────────────────────────────────

#[test]
fn parse_plain_path() {
    let (file, symbol) = parse_entrypoint("src/main.mts");
    assert_eq!(file, PathBuf::from("src/main.mts"));
    assert!(symbol.is_none());
}

#[test]
fn parse_path_with_symbol() {
    let (file, symbol) = parse_entrypoint("src/queues.mts#enqueueBulkTopicEmbeddings");
    assert_eq!(file, PathBuf::from("src/queues.mts"));
    assert_eq!(symbol.as_deref(), Some("enqueueBulkTopicEmbeddings"));
}

#[test]
fn parse_path_multiple_hashes_splits_on_first() {
    let (_file, symbol) = parse_entrypoint("src/foo.mts#sym#extra");
    assert_eq!(symbol.as_deref(), Some("sym#extra"));
}

#[test]
fn workflow_virtual_entrypoint_suffixes_round_trip() {
    let file = Path::new("/repo/.github/workflows/main.yml");
    assert_eq!(
        workflow_node_from_suffix(file, "job:build"),
        Some(NodeId::WorkflowJob {
            workflow_file: file.to_path_buf(),
            job: "build".to_string(),
        })
    );
    assert_eq!(
        workflow_node_from_suffix(file, "job:build/step:3"),
        Some(NodeId::WorkflowStep {
            workflow_file: file.to_path_buf(),
            job: "build".to_string(),
            step: 3,
        })
    );
    assert!(workflow_node_from_suffix(file, "job:/step:3").is_none());
    assert!(workflow_node_from_suffix(file, "job:build/step:nope").is_none());
    assert!(workflow_node_from_suffix(file, "ordinary-symbol").is_none());
}

#[test]
fn resolve_entrypoints_promotes_workflow_suffixes_to_virtual_nodes() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/workflow-topology"),
    );
    let entrypoints = resolve_entrypoints(
        &[PathBuf::from(".github/workflows/main.yml#job:build/step:0")],
        &root,
        &root,
    );

    assert_eq!(entrypoints[0].symbol, None);
    assert_eq!(
        entrypoints[0].node,
        NodeId::WorkflowStep {
            workflow_file: root.join(".github/workflows/main.yml"),
            job: "build".to_string(),
            step: 0,
        }
    );
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
    assert_eq!(
        entrypoints[1].node,
        graph::NodeId::File(cwd.join("does-not-exist.mts"))
    );
    assert_eq!(entrypoints[1].symbol, None);
    assert_eq!(
        entrypoints[2].file,
        crate::codebase::ts_resolver::normalize_path(&cwd.join("../../other.mts"))
    );
    assert_eq!(entrypoints[2].symbol.as_deref(), Some("exportName"));
}

#[test]
fn resolve_entrypoints_infers_workspace_package_directory_entry() {
    let root = fixture_root("graph-modules");
    let args = parse(&["deps", "packages/local"]);
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);

    assert_eq!(
        entrypoints[0].node,
        graph::NodeId::File(root.join("packages/local/src/index.mts"))
    );
    assert_eq!(
        entrypoints[0].file,
        root.join("packages/local/src/index.mts")
    );
}

#[test]
fn resolve_entrypoints_infers_plain_directory_index_entry() {
    let root = fixture_root("graph-entrypoint-dir");
    let args = parse(&["deps", "."]);
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);

    assert_eq!(
        entrypoints[0].node,
        graph::NodeId::File(root.join("src/index.ts"))
    );
    assert_eq!(entrypoints[0].file, root.join("src/index.ts"));
}

#[test]
fn resolve_entrypoints_infers_plain_directory_cjs_index_entry() {
    let root = fixture_root("graph-entrypoint-dir-cjs");
    let args = parse(&["deps", "."]);
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);

    assert_eq!(
        entrypoints[0].node,
        graph::NodeId::File(root.join("index.cjs"))
    );
    assert_eq!(entrypoints[0].file, root.join("index.cjs"));
}

#[test]
fn resolve_entrypoints_keeps_directory_without_entry_as_file_node() {
    let root = fixture_root("graph-empty-dir");
    let args = parse(&["deps", "empty"]);
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);

    assert_eq!(entrypoints[0].node, graph::NodeId::File(root.join("empty")));
    assert_eq!(entrypoints[0].file, root.join("empty"));
}

#[test]
fn resolve_entrypoints_accepts_workspace_package_specifier() {
    let root = fixture_root("graph-modules");
    let args = parse(&["deps", "@local/pkg"]);
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);

    assert_eq!(
        entrypoints[0].node,
        graph::NodeId::File(root.join("packages/local/src/index.mts"))
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

    assert_eq!(
        entrypoints[0].node,
        graph::NodeId::Module("@external/pkg".to_string())
    );
    assert_eq!(entrypoints[0].symbol.as_deref(), Some("handler"));
}

#[test]
fn resolve_entrypoints_keeps_package_subpath_with_extension_as_module_node() {
    let root = fixture_root("graph-modules");
    let args = parse(&["dependents", "lodash", "lodash/fp.js"]);
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);

    assert_eq!(
        entrypoints[0].node,
        graph::NodeId::Module("lodash".to_string())
    );
    assert_eq!(
        entrypoints[1].node,
        graph::NodeId::Module("lodash/fp.js".to_string())
    );
}

#[test]
fn resolve_entrypoints_treats_missing_source_path_with_existing_parent_as_file_node() {
    let root = fixture_root("graph-modules");
    let args = parse(&["dependents", "src/new-file.ts"]);
    let entrypoints = resolve_entrypoints(&args.files, &root, &root);

    assert_eq!(
        entrypoints[0].node,
        graph::NodeId::File(root.join("src/new-file.ts"))
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
        graph::NodeId::File(fixture.path().join("explicit-dir"))
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
    assert!(
        !root_dependency_names(&malformed_root, malformed_files.all())
            .contains("lodash")
    );
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
