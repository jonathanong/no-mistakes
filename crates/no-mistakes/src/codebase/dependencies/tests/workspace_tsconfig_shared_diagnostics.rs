use super::*;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/workspace-resolution"),
    )
}

fn import_args(root: PathBuf, file: &str) -> TraverseArgs {
    TraverseArgs {
        files: vec![PathBuf::from(file)],
        file_symbols: Vec::new(),
        file_entrypoints_are_structured: Vec::new(),
        root: Some(root),
        tsconfig: None,
        depth: Some(3),
        filters: Vec::new(),
        target_modules: Vec::new(),
        tests: Vec::new(),
        relationships: vec![RelationshipArg::Import],
        include_symbols: false,
        format: Some(Format::Json),
        json: false,
        timings: false,
    }
}

#[test]
fn shared_traversal_does_not_repeat_lazy_tsconfig_diagnostics_in_later_reports() {
    let root = workspace_root();
    let cwd = std::env::current_dir().unwrap();
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
    let first = collect_and_filter_entries_shared(
        &import_args(root.clone(), "apps/ambiguous/src/entry.ts"),
        Direction::Deps,
        &cwd,
        &mut shared,
    )
    .unwrap();
    assert!(first.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == crate::codebase::ts_resolver::TsConfigDiagnosticKind::AmbiguousOwnership
    }));

    let second = collect_and_filter_entries_shared(
        &import_args(root.clone(), "apps/web/src/entry.ts"),
        Direction::Deps,
        &cwd,
        &mut shared,
    )
    .unwrap();
    assert!(!second.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == crate::codebase::ts_resolver::TsConfigDiagnosticKind::AmbiguousOwnership
    }));

    let third = collect_and_filter_entries_shared(
        &import_args(root, "apps/ambiguous/src/entry.ts"),
        Direction::Deps,
        &cwd,
        &mut shared,
    )
    .unwrap();
    assert!(third.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == crate::codebase::ts_resolver::TsConfigDiagnosticKind::AmbiguousOwnership
    }));
}

#[test]
fn shared_traversal_replays_cached_graph_diagnostics_without_leaking_them() {
    let root = workspace_root();
    let cwd = std::env::current_dir().unwrap();
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
    let mut web = import_args(root.clone(), "apps/web/src/entry.ts");
    // A full graph sees the separate ambiguous project; the web entry itself
    // has unambiguous ownership, so this diagnoses graph-build cache replay.
    web.relationships = vec![RelationshipArg::All];

    let first =
        collect_and_filter_entries_shared(&web, Direction::Deps, &cwd, &mut shared).unwrap();
    assert!(first.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == crate::codebase::ts_resolver::TsConfigDiagnosticKind::AmbiguousOwnership
    }));

    let replayed =
        collect_and_filter_entries_shared(&web, Direction::Deps, &cwd, &mut shared).unwrap();
    assert!(replayed.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == crate::codebase::ts_resolver::TsConfigDiagnosticKind::AmbiguousOwnership
    }));

    let mut worker = import_args(root, "services/worker/src/entry.ts");
    worker.relationships = vec![RelationshipArg::All];
    let distinct =
        collect_and_filter_entries_shared(&worker, Direction::Deps, &cwd, &mut shared).unwrap();
    assert!(!distinct.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == crate::codebase::ts_resolver::TsConfigDiagnosticKind::AmbiguousOwnership
    }));
}
