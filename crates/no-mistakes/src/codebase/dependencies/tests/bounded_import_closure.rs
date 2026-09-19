use serde_json::Value;
use std::path::PathBuf;

fn bounded_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/bounded-import-closure"),
    )
}

fn import_forms_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/import-forms/fixture"),
    )
}

fn oxc_reexport_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/oxc-export-from-reexport"),
    )
}

fn mixed_type_import_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/mixed-type-import/fixture"),
    )
}

fn workspace_resolution_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/workspace-resolution"),
    )
}

fn paths_precedence_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/paths-precedence"),
    )
}

fn import_only_args(root: PathBuf, files: Vec<PathBuf>) -> TraverseArgs {
    TraverseArgs {
        files,
        root: Some(root),
        relationships: vec![
            RelationshipArg::ImportStatic,
            RelationshipArg::ImportDynamic,
            RelationshipArg::ImportType,
        ],
        format: Some(Format::Json),
        json: true,
        ..Default::default()
    }
}

fn assert_compact_matches_unbounded(root: PathBuf, file: &str) {
    let mut unbounded = import_only_args(root, vec![PathBuf::from(file)]);
    let unbounded_value: Value =
        serde_json::from_str(&run_json(unbounded.clone(), Direction::Deps).unwrap()).unwrap();
    unbounded.candidate_include = vec!["**/*".to_string()];
    unbounded.projection = TraverseProjection::Paths;
    let bounded_value: Value =
        serde_json::from_str(&run_json(unbounded, Direction::Deps).unwrap()).unwrap();
    assert_eq!(
        sorted_graph_paths(&unbounded_value),
        closure_paths(&bounded_value)
    );
}

fn bounded_args(root: PathBuf, files: Vec<PathBuf>) -> TraverseArgs {
    TraverseArgs {
        files,
        root: Some(root),
        relationships: vec![
            RelationshipArg::ImportStatic,
            RelationshipArg::ImportDynamic,
            RelationshipArg::ImportType,
        ],
        candidate_include: vec!["web/**".to_string()],
        candidate_exclude: vec!["**/*.test.*".to_string()],
        projection: TraverseProjection::Paths,
        format: Some(Format::Json),
        json: true,
        ..Default::default()
    }
}

fn closure_paths(value: &Value) -> Vec<String> {
    value["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| {
            entry
                .as_str()
                .map(str::to_string)
                .or_else(|| entry["path"].as_str().map(str::to_string))
        })
        .collect()
}

fn sorted_graph_paths(value: &Value) -> Vec<String> {
    let mut paths = value["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry["path"].as_str().map(str::to_string))
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

#[test]
fn candidate_include_flag_is_repeatable() {
    let args = TraverseArgs::parse_from([
        "deps",
        "web/app/page.tsx",
        "--candidate-include",
        "web/**",
        "--candidate-exclude",
        "**/*.test.*",
        "--projection",
        "paths",
        "--relationship",
        "import-static",
    ]);
    assert_eq!(args.candidate_include, vec!["web/**"]);
    assert_eq!(args.candidate_exclude, vec!["**/*.test.*"]);
    assert_eq!(args.projection, TraverseProjection::Paths);
}

#[test]
fn candidate_bounds_reject_dependents() {
    let args = bounded_args(bounded_root(), vec![PathBuf::from("web/app/page.tsx")]);
    let err = validate_candidate_bounds(&args, Direction::Dependents).unwrap_err();
    assert!(format!("{err}").contains("dependencies"));
}

#[test]
fn candidate_bounds_reject_symbols_and_non_import_relationships() {
    let mut args = bounded_args(bounded_root(), vec![PathBuf::from("web/app/page.tsx")]);
    args.include_symbols = true;
    assert!(validate_candidate_bounds(&args, Direction::Deps).is_err());
    args.include_symbols = false;
    args.relationships = vec![RelationshipArg::Workspace];
    assert!(validate_candidate_bounds(&args, Direction::Deps).is_err());
}

#[test]
fn bounded_inventory_excludes_unrelated_tests_and_escapes_workspace_paths() {
    let root = bounded_root();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        run_json(
            bounded_args(root.clone(), vec![PathBuf::from("web/app/page.tsx")]),
            Direction::Deps,
        )
        .unwrap()
    };
    let value: Value = serde_json::from_str(&output).unwrap();
    let files = closure_paths(&value);
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("packages/ui/src/button.ts")),
        "{files:?}"
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("packages/data/src/registry.ts")),
        "{files:?}"
    );
    assert!(
        !files.iter().any(|path| path.contains("page.test")),
        "{files:?}"
    );
    assert!(
        !files.iter().any(|path| path.contains("unrelated.test")),
        "{files:?}"
    );
    assert!(value.get("roots").is_none(), "{value}");
    assert!(value.get("tsconfig_provenance").is_none(), "{value}");
    assert_eq!(
        value.as_object().unwrap().keys().collect::<Vec<_>>(),
        ["files", "diagnostics"]
    );

    let work = observer.snapshot().work;
    assert!(work["graph.candidate_excluded"] >= 2, "{work:#?}");
    assert!(work["graph.candidate_files"] >= 1, "{work:#?}");
    assert!(
        work["parse.files"] >= 1 && work["parse.files"] <= 4,
        "{work:#?}"
    );
    let inventory =
        CandidateInventory::new(&["web/**".to_string()], &["**/*.test.*".to_string()]).unwrap();
    assert!(!inventory.matches(&root, &root.join("web/app/page.test.tsx")));
    assert!(!inventory.matches(&root, &root.join("web/lib/unrelated.test.ts")));
    assert!(inventory.matches(&root, &root.join("web/app/page.tsx")));
}

#[test]
fn compact_projection_does_not_change_reachable_paths() {
    let root = bounded_root();
    let mut graph_args = bounded_args(root.clone(), vec![PathBuf::from("web/app/page.tsx")]);
    graph_args.projection = TraverseProjection::Graph;
    let graph: Value =
        serde_json::from_str(&run_json(graph_args, Direction::Deps).unwrap()).unwrap();
    let compact: Value = serde_json::from_str(
        &run_json(
            bounded_args(root.clone(), vec![PathBuf::from("web/app/page.tsx")]),
            Direction::Deps,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(sorted_graph_paths(&graph), closure_paths(&compact));
    assert!(graph.get("roots").is_some());
    assert!(compact.get("roots").is_none());
    let again: Value = serde_json::from_str(
        &run_json(
            bounded_args(root, vec![PathBuf::from("web/app/page.tsx")]),
            Direction::Deps,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(compact, again);
}

#[test]
fn omitted_projection_keeps_graph_shape() {
    let root = bounded_root();
    let mut omitted = bounded_args(root.clone(), vec![PathBuf::from("web/app/page.tsx")]);
    omitted.projection = TraverseProjection::Graph;
    let explicit = omitted.clone();
    // Default clap value is Graph; an omitted field must match an explicit graph.
    omitted.projection = TraverseProjection::default();
    let omitted: Value =
        serde_json::from_str(&run_json(omitted, Direction::Deps).unwrap()).unwrap();
    let explicit: Value =
        serde_json::from_str(&run_json(explicit, Direction::Deps).unwrap()).unwrap();
    assert_eq!(omitted, explicit);
    assert!(omitted.get("roots").is_some());
    assert!(omitted["files"].as_array().unwrap()[0]
        .get("path")
        .is_some());
}

#[test]
fn bounded_inventory_does_not_parse_excluded_tests() {
    let root = bounded_root();
    let cwd = std::env::current_dir().unwrap();
    crate::ast::begin_parse_count(&root);
    let mut shared = SharedTraversalContext::prepare(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan {
            imports: true,
            ..Default::default()
        },
    )
    .unwrap();
    let args = bounded_args(root.clone(), vec![PathBuf::from("web/app/page.tsx")]);
    shared.apply_candidate_inventory(&args).unwrap();
    collect_and_filter_entries_shared(&args, Direction::Deps, &cwd, &mut shared).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    assert!(
        !counts.contains_key(&root.join("web/app/page.test.tsx")),
        "{counts:#?}"
    );
    assert!(
        !counts.contains_key(&root.join("web/lib/unrelated.test.ts")),
        "{counts:#?}"
    );
    assert!(
        counts.contains_key(&root.join("web/app/page.tsx")),
        "{counts:#?}"
    );
}

#[test]
fn bounded_closure_matches_unbounded_import_forms_paths() {
    assert_compact_matches_unbounded(import_forms_root(), "static.mts");
}

#[test]
fn bounded_closure_matches_unbounded_reexport_paths() {
    let root = oxc_reexport_root();
    let mut unbounded = import_only_args(root, vec![PathBuf::from("consumer.mts")]);
    unbounded.relationships = vec![RelationshipArg::ImportStatic];
    let unbounded_value: Value =
        serde_json::from_str(&run_json(unbounded.clone(), Direction::Deps).unwrap()).unwrap();
    unbounded.candidate_include = vec!["**/*".to_string()];
    unbounded.projection = TraverseProjection::Paths;
    let bounded_value: Value =
        serde_json::from_str(&run_json(unbounded, Direction::Deps).unwrap()).unwrap();
    assert_eq!(
        sorted_graph_paths(&unbounded_value),
        closure_paths(&bounded_value)
    );
}

#[test]
fn bounded_closure_matches_unbounded_mixed_type_import_paths() {
    assert_compact_matches_unbounded(mixed_type_import_root(), "importer.mts");
}

#[test]
fn bounded_closure_matches_unbounded_workspace_resolution_paths() {
    assert_compact_matches_unbounded(workspace_resolution_root(), "apps/web/src/entry.ts");
}

#[test]
fn bounded_closure_matches_unbounded_path_alias_paths() {
    assert_compact_matches_unbounded(paths_precedence_root(), "src/entry.ts");
}

#[test]
fn bounded_finite_depth_does_not_parse_the_full_closure() {
    let root = bounded_root();
    crate::ast::begin_parse_count(&root);
    let mut args = bounded_args(root.clone(), vec![PathBuf::from("web/app/page.tsx")]);
    args.depth = Some(0);
    run_json(args, Direction::Deps).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    assert!(
        !counts.contains_key(&root.join("packages/ui/src/button.ts")),
        "{counts:#?}"
    );
    assert!(
        !counts.contains_key(&root.join("packages/data/src/registry.ts")),
        "{counts:#?}"
    );
}

#[test]
fn bounded_package_dir_root_resolves_outside_candidate_include() {
    let root = bounded_root();
    let mut args = bounded_args(root, vec![PathBuf::from("packages/ui")]);
    args.candidate_include = vec!["web/**".to_string()];
    args.projection = TraverseProjection::Graph;
    let value: Value = serde_json::from_str(&run_json(args, Direction::Deps).unwrap()).unwrap();
    assert_eq!(
        value["tsconfig_provenance"][0]["importer"],
        "packages/ui/src/button.ts",
        "{value}"
    );
}

#[test]
fn bounded_compact_projection_keeps_ambiguous_ownership_diagnostics() {
    let mut args = import_only_args(
        workspace_resolution_root(),
        vec![PathBuf::from("apps/ambiguous/src/entry.ts")],
    );
    args.candidate_include = vec!["**/*".to_string()];
    args.projection = TraverseProjection::Paths;
    let value: Value = serde_json::from_str(&run_json(args, Direction::Deps).unwrap()).unwrap();
    assert!(
        value["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["kind"] == "ambiguous-ownership"),
        "{value}"
    );
}
