use super::*;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn markdown_and_cargo_edge_collectors_reuse_prepared_source_store() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("codebase-intel"));
    let readme = root.join("README.md");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let inventory = Arc::new(FileInventory::from_paths(&files));
    let store = SourceStore::new(inventory);
    let graph_files = GraphFiles::from_files(files.clone());
    let interner = crate::codebase::analysis_session::PathInterner::new();

    let md_edges = collect_md_edges(
        std::slice::from_ref(&readme),
        &graph_files,
        &interner,
        Some(&store),
    );
    assert!(md_edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::MarkdownLink));
    let after_md = store.physical_read_count();
    assert!(
        after_md >= 1,
        "markdown collectors must read through the prepared SourceStore"
    );

    let bins = collect_cargo_bins(&root, &files, Some(&store));
    assert!(!bins.by_name.is_empty() || files.iter().any(|path| path.ends_with("Cargo.toml")));
    let after_bins = store.physical_read_count();
    assert!(
        after_bins >= after_md,
        "cargo collectors must read through the prepared SourceStore"
    );

    let _ = collect_md_edges(
        std::slice::from_ref(&readme),
        &graph_files,
        &interner,
        Some(&store),
    );
    let _ = collect_cargo_bins(&root, &files, Some(&store));
    assert_eq!(
        store.physical_read_count(),
        after_bins,
        "markdown and cargo collectors must reuse the prepared SourceStore"
    );
}

#[test]
fn cargo_bin_collector_skips_unreadable_and_out_of_root_manifests() {
    let ts_root = crate::codebase::ts_resolver::normalize_path(&fixture("unique-exports-basic"));
    assert!(
        collect_cargo_bins(&ts_root, &[ts_root.join("Cargo.toml")], None)
            .by_name
            .is_empty(),
        "a listed root Cargo.toml that cannot be read must yield no bins"
    );

    let workspace = crate::codebase::ts_resolver::normalize_path(&fixture("cargo-workspace-ci"));
    let bins = collect_cargo_bins(
        &workspace,
        &[
            workspace.join("Cargo.toml"),
            workspace.join("crates/missing-member/Cargo.toml"),
            PathBuf::from("/outside/Cargo.toml"),
        ],
        None,
    );
    assert!(
        !bins.by_name.contains_key("outside"),
        "manifests outside the workspace root must not contribute cargo bins"
    );
}

#[test]
fn graph_helpers_require_facts_and_playwright_snapshots() {
    let err = parsed_imports_for_plan(
        GraphBuildPlan {
            imports: true,
            ..GraphBuildPlan::default()
        },
        &[],
        None,
    )
    .unwrap_err();
    assert!(err
        .to_string()
        .contains("TS import facts are required when import, workspace, or asset edges are requested"));
    let playwright_err = match require_playwright_route_snapshot(None) {
        Ok(_) => panic!("missing Playwright snapshot must fail"),
        Err(error) => error,
    };
    assert!(playwright_err
        .to_string()
        .contains("Playwright graph plan requires a visible-path snapshot"));
}
