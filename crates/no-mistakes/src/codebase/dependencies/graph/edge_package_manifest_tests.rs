use super::*;

    #[test]
    fn package_edges_reuse_workspace_manifest_metadata_without_rereading() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/large-graph-monorepo/fixture");
        let files = crate::codebase::ts_source::discover_visible_paths(&root);
        let inventory = std::sync::Arc::new(
            crate::codebase::ts_source::FileInventory::from_paths(&files),
        );
        let sources = crate::codebase::ts_source::SourceStore::new(inventory);
        let workspace =
            crate::codebase::workspaces::load_indexed_from_source_store(&root, &sources).unwrap();
        let reads_after_workspace_load = sources.physical_read_count();
        let graph_files = GraphFiles::from_files(files.clone());

        let edges = collect_workspace_manifest_edges(&files, &workspace, &graph_files);

        assert!(edges
            .iter()
            .any(|(_, _, kind)| *kind == EdgeKind::PackageDependency));
        assert_eq!(sources.physical_read_count(), reads_after_workspace_load);
    }
