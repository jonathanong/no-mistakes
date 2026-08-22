use super::*;

#[test]
fn package_edges_reuse_workspace_manifest_metadata_without_rereading() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/large-graph-monorepo/fixture");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
        &files,
    ));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    let workspace =
        crate::codebase::workspaces::load_indexed_from_source_store(&root, &sources).unwrap();
    let reads_after_workspace_load = sources.physical_read_count();
    let graph_files = GraphFiles::from_files(files.clone());

    let edges = collect_workspace_manifest_edges(
        &files,
        &workspace,
        &graph_files,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    assert!(edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::PackageDependency));
    assert_eq!(sources.physical_read_count(), reads_after_workspace_load);
}

#[cfg(unix)]
#[test]
fn workspace_manifest_edges_remap_canonical_entries_to_visible_paths() {
    let via_link = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link/src/value.ts"),
    );
    let via_real = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/real/src/value.ts"),
    );
    let manifest = PathBuf::from("/workspace/app/package.json");
    let graph_files = GraphFiles::from_files(vec![via_link.clone()]);
    assert!(!graph_files.contains_visible(&via_real));
    assert_eq!(graph_files.visible_path(&via_real), Some(via_link.as_path()));

    let workspace = crate::codebase::workspaces::IndexedWorkspaceMap::from_packages(vec![
        crate::codebase::workspaces::WorkspacePackage {
            name: "@x/linked".to_string(),
            dir: via_real.parent().unwrap().to_path_buf(),
            entry: Some(via_real),
            exports: None,
            imports: None,
        },
    ])
    .with_manifest_dependency_names(manifest.clone(), vec!["@x/linked".to_string()]);

    let edges = collect_workspace_manifest_edges(
        std::slice::from_ref(&manifest),
        &workspace,
        &graph_files,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    assert!(edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::PackageDependency
            && from.as_file() == Some(manifest.as_path())
            && to.as_file() == Some(via_link.as_path())
    }));
}
