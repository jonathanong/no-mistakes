use super::*;
use crate::codebase::ts_source::{FileInventory, SourceStore};
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
