use super::*;
use crate::codebase::ts_resolver::VisiblePathLookup;

#[test]
fn snapshot_aliases_never_admit_unselected_files_or_directories() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("bounded-import-closure"));
    let admitted = root.join("web/app/page.tsx");
    let excluded = root.join("web/lib/unrelated.test.ts");
    let graph_files = GraphFiles::from_files(vec![admitted.clone()]);
    let universe = GraphFiles::from_files(vec![admitted.clone()]);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let overlay = SnapshotResolutionVisible::new(&graph_files, &universe, &snapshot, &root);
    assert_eq!(overlay.visible_alias(&excluded), None);
    assert_eq!(
        overlay.visible_alias(&root.join("web/lib/../lib/unrelated.test.ts")),
        None
    );
    assert_eq!(overlay.visible_alias(&root.join("web/lib")), None);
    assert_eq!(overlay.visible_alias(&root.join("absent.ts")), None);
    assert_eq!(overlay.visible_alias(&admitted), Some(admitted.clone()));
    assert_eq!(overlay.visible_cache_key(), vec![admitted]);
}

#[test]
fn normalized_import_escape_uses_the_overlay_without_widening_graph_membership() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("bounded-import-closure"));
    let admitted = root.join("packages/ui/src/button.ts");
    let target = root.join("packages/ui/src/../src/button.ts");
    let graph_files = GraphFiles::from_files(vec![]);
    let visible = std::collections::HashSet::from([admitted.clone()]);
    assert_eq!(visible.visible_alias(&target), None);
    assert_eq!(
        visible_or_escaped_path(&graph_files, Some(&visible), &target),
        Some(admitted)
    );
    assert!(!graph_files.contains_visible(&target));
    assert_eq!(graph_files.visible_len(), 0);
    assert_eq!(
        visible_or_escaped_path(&graph_files, Some(&visible), &root.join("web/app/page.tsx")),
        None
    );
}

#[cfg(unix)]
#[test]
fn snapshot_canonical_aliases_preserve_the_saved_symlink_namespace() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link"),
    );
    let lexical = root.join("src/value.ts");
    let canonical = crate::codebase::ts_resolver::normalize_path(&lexical.canonicalize().unwrap());
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(
        &root,
        std::slice::from_ref(&lexical),
    );
    assert_ne!(lexical, canonical);
    let canonical_root = root.canonicalize().unwrap();
    let canonical_snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(
        &canonical_root,
        std::slice::from_ref(&canonical),
    );
    assert_eq!(
        snapshot_visible_path(&canonical_snapshot, &canonical_root, &lexical),
        Some(canonical.clone())
    );
    assert_eq!(
        snapshot_visible_path(&snapshot, &root, &canonical),
        Some(lexical.clone())
    );
    assert_eq!(
        snapshot_visible_path(&snapshot, &root, &root.join("src/../src/value.ts")),
        Some(lexical)
    );
    let outside = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/bounded-import-closure/web/app/page.tsx"),
    );
    assert_eq!(snapshot_visible_path(&snapshot, &root, &outside), None);
}
