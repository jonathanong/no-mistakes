use super::*;
use crate::codebase::ts_resolver::VisiblePathLookup;
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
        candidate_include: Vec::new(),
        candidate_exclude: Vec::new(),
        projection: TraverseProjection::Graph,
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

fn work_count(observer: &crate::diagnostics::InvocationObserver, metric: &str) -> u64 {
    observer.snapshot().work.get(metric).copied().unwrap_or(0)
}

#[test]
fn explicit_root_outside_the_universe_reuses_one_bounded_seed_projection() {
    let root = lazy_root();
    // Outside the discovered universe. The bounded seed adds it only to the
    // walk's graph files, so the overlay is universe ∪ this root.
    let outside = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/signature-impact/utils.mts"),
    );
    let cwd = std::env::current_dir().unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let allowed = relationship_filter(&[RelationshipArg::ImportStatic]);
    let mut shared = SharedTraversalContext::prepare_with_session(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan::from_allowed(allowed.as_ref()),
        session,
    )
    .unwrap();
    let args = import_args(root.clone(), vec![outside.clone()]);
    assert!(!shared.resolution_universe().contains_visible(&outside));
    shared
        .seed_bounded_lazy_import_graph_from_args(&args, &cwd)
        .unwrap();
    let graph = shared
        .bounded_lazy_import_graph(&args)
        .expect("bounded seed keeps the explicit root");
    assert!(graph.contains_file(&outside));

    let mut graph_files = shared.candidate_graph_files(&args).unwrap();
    for path in explicit_existing_entry_files(&args, &shared.root, &cwd) {
        graph_files.add_explicit_root(&path);
    }
    assert!(graph_files.contains_visible(&outside));
    let overlay = shared.snapshot_resolution_visible(&graph_files);
    let mut visited = 0usize;
    overlay.visit_visible(&mut |_| visited += 1);
    assert_eq!(overlay.visible_len(), visited);
    assert!(overlay.visible_len() > shared.resolution_universe().visible_len());

    let before = work_count(&observer, "resolver.visible_normalizations");
    let scopes_before = work_count(&observer, "resolver.scope_constructions");
    assert!(before >= 1);
    let normalized = shared.session.normalized_visible(&overlay);
    assert!(normalized.contains_visible(&outside));
    assert!(normalized.contains_visible(&root.join("src/a.mts")));
    let again = shared.session.normalized_visible(&overlay);
    assert!(std::sync::Arc::ptr_eq(&normalized, &again));
    assert_eq!(
        work_count(&observer, "resolver.visible_normalizations"),
        before,
        "an explicit root outside the universe must reuse the seeded projection"
    );
    assert_eq!(
        work_count(&observer, "resolver.scope_constructions"),
        scopes_before
    );
}
