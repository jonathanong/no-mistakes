use super::*;
use std::path::PathBuf;

fn lazy_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/lazy-import/fixture"),
    )
}

fn import_args(root: PathBuf, files: Vec<PathBuf>) -> TraverseArgs {
    TraverseArgs {
        files,
        file_symbols: Vec::new(),
        file_entrypoints_are_structured: Vec::new(),
        root: Some(root),
        tsconfig: None,
        depth: None,
        filters: Vec::new(),
        target_modules: Vec::new(),
        tests: Vec::new(),
        relationships: vec![RelationshipArg::ImportStatic],
        include_symbols: false,
        format: Some(Format::Json),
        json: false,
        timings: false,
    }
}

fn prepare_lazy(root: PathBuf) -> SharedTraversalContext {
    let allowed = relationship_filter(&[RelationshipArg::ImportStatic]);
    SharedTraversalContext::prepare(
        root,
        None,
        None,
        graph::GraphBuildPlan::from_allowed(allowed.as_ref()),
    )
    .unwrap()
}

#[test]
fn seed_lazy_import_graph_is_idempotent() {
    let root = lazy_root();
    let cwd = std::env::current_dir().unwrap();
    let mut shared = prepare_lazy(root.clone());
    let args = import_args(root.clone(), vec![root.join("src/a.mts")]);
    shared
        .seed_lazy_import_graph_from_args(&args, &cwd)
        .unwrap();
    assert!(shared.lazy_import_graph().is_some());
    shared
        .seed_lazy_import_graph_from_args(&args, &cwd)
        .unwrap();
    assert!(shared.lazy_import_graph().is_some());
}

#[test]
fn seed_lazy_import_graph_skips_missing_and_empty_roots() {
    let root = lazy_root();
    let cwd = std::env::current_dir().unwrap();
    let mut shared = prepare_lazy(root.clone());
    shared
        .seed_lazy_import_graph_from_args(&import_args(root.clone(), Vec::new()), &cwd)
        .unwrap();
    assert!(shared.lazy_import_graph().is_none());
    shared
        .seed_lazy_import_graph_from_args(
            &import_args(root.clone(), vec![root.join("src/missing.mts")]),
            &cwd,
        )
        .unwrap();
}
