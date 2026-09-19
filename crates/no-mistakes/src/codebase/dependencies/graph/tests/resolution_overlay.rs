#[test]
fn visible_or_escaped_path_without_overlay_stays_inside_graph_files() {
    let path = PathBuf::from("/repo/src/a.ts");
    let graph_files = GraphFiles::from_files(vec![path.clone()]);
    assert_eq!(
        visible_or_escaped_path(&graph_files, None, &path),
        Some(path)
    );
    assert_eq!(
        visible_or_escaped_path(&graph_files, None, Path::new("/repo/src/missing.ts")),
        None
    );
}

#[test]
fn snapshot_overlay_exposes_escaped_universe_paths() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/bounded-import-closure"),
    );
    let escaped = root.join("packages/ui/src/button.ts");
    let snapshot_only = root.join("web/lib/unrelated.test.ts");
    let graph_files = GraphFiles::from_files(vec![root.join("web/app/page.tsx")]);
    let universe = GraphFiles::from_files(vec![
        root.join("web/app/page.tsx"),
        escaped.clone(),
    ]);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let overlay = SnapshotResolutionVisible::new(&graph_files, &universe, &snapshot, &root);
    assert!(overlay.contains_visible(&escaped));
    assert!(!overlay.contains_visible(&snapshot_only));
    assert_eq!(overlay.visible_alias(&escaped).as_deref(), Some(escaped.as_path()));
    assert_eq!(overlay.visible_len(), universe.visible_len());
    assert!(overlay.visible_cache_key().contains(&escaped));
    assert!(!overlay.visible_cache_key().contains(&snapshot_only));
    let ghost = PathBuf::from("/not-in-snapshot/ghost.ts");
    let graph_files = GraphFiles::from_files(vec![ghost.clone()]);
    let overlay = SnapshotResolutionVisible::new(&graph_files, &universe, &snapshot, &root);
    assert!(overlay.visible_cache_key().contains(&ghost));
    assert_eq!(
        visible_or_escaped_path(&graph_files, Some(&overlay), &escaped),
        Some(escaped)
    );
}

#[cfg(unix)]
#[test]
fn snapshot_overlay_remaps_canonical_targets_to_the_symlink_namespace() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link"),
    );
    let lexical = root.join("src/value.ts");
    let canonical = crate::codebase::ts_resolver::normalize_path(&lexical.canonicalize().unwrap());
    assert_ne!(canonical, lexical);
    let graph_files = GraphFiles::from_files(vec![root.join("src/entry.ts")]);
    let universe = GraphFiles::from_files(vec![root.join("src/entry.ts"), lexical.clone()]);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let overlay = SnapshotResolutionVisible::new(&graph_files, &universe, &snapshot, &root);
    assert_eq!(
        overlay.visible_alias(&canonical).as_deref(),
        Some(lexical.as_path())
    );
    assert_eq!(
        visible_or_escaped_path(&graph_files, Some(&overlay), &canonical),
        Some(lexical)
    );
}
