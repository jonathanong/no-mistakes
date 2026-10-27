//! Git-index classification tests. This file lives under `tests/` so
//! `rust-max-lines-per-file` applies the test-file cap.
use super::super::FileInventory;
use std::path::PathBuf;

fn fixture(path: &str) -> PathBuf {
    super::fixture(path)
}

fn saved_ts_source(name: &str) -> tempfile::TempDir {
    crate::test_support::materialize_saved_fixture(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/ts-source")
            .join(name),
    )
}

#[test]
fn git_tracked_regular_files_skip_inventory_metadata_stats() {
    let dir = saved_ts_source("git-index-classify");
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(dir.path());
    let sources = snapshot.source_store_for(dir.path());
    let inventory = sources.inventory();

    assert!(
        inventory.len() >= 3,
        "git-index-classify fixture must contribute tracked files"
    );
    assert!(inventory.paths().iter().all(|path| inventory
        .classification_for_path(path)
        .unwrap()
        .is_lexical_file()));
    assert_eq!(
        inventory.metadata_stat_count(),
        0,
        "tracked regular files must be classified from git index mode"
    );

    let restated = FileInventory::from_paths(inventory.paths().as_slice());
    assert_eq!(restated.paths(), inventory.paths());
    assert!(
        restated.metadata_stat_count() >= inventory.len(),
        "from_paths must still stat unclassified paths"
    );
}

#[cfg(unix)]
#[test]
fn git_index_mode_keeps_unstaged_symlink_replacement_as_regular_file() {
    let dir = saved_ts_source("git-index-classify");
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    let path = dir.path().join("staged-regular.mts");
    std::fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink("regular.mts", &path).unwrap();

    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(dir.path());
    let kind = snapshot
        .classification_for(dir.path(), &path)
        .expect("unstaged type-change must remain visible");
    assert!(kind.is_lexical_file());
    assert!(!kind.is_lexical_symlink());
    assert!(kind.target_is_file());
    assert_eq!(
        snapshot
            .source_store_for(dir.path())
            .inventory()
            .metadata_stat_count(),
        0
    );
}

#[cfg(unix)]
#[test]
fn git_tracked_symlink_still_consults_worktree_metadata() {
    let dir = saved_ts_source("git-index-classify");
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    let link = dir.path().join("link.mts");
    std::os::unix::fs::symlink("regular.mts", &link).unwrap();
    crate::test_support::git_add_all(dir.path());

    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(dir.path());
    let sources = snapshot.source_store_for(dir.path());
    let inventory = sources.inventory();
    let kind = inventory.classification_for_path(&link).unwrap();
    assert!(!kind.is_lexical_file());
    assert!(kind.is_lexical_symlink());
    assert!(kind.target_is_file());
    assert_eq!(inventory.metadata_stat_count(), 1);
}

#[cfg(unix)]
#[test]
fn git_tracked_symlink_replaced_by_regular_file_uses_worktree_type() {
    let dir = saved_ts_source("git-index-classify");
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    let link = dir.path().join("link.mts");
    std::os::unix::fs::symlink("regular.mts", &link).unwrap();
    crate::test_support::git_add_all(dir.path());
    std::fs::remove_file(&link).unwrap();
    // Copy a tracked regular file over the symlink. The replacement has to
    // happen after git add, and the assertions only observe the worktree type.
    std::fs::copy(dir.path().join("regular.mts"), &link).unwrap();

    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(dir.path());
    let sources = snapshot.source_store_for(dir.path());
    let inventory = sources.inventory();
    let kind = inventory.classification_for_path(&link).unwrap();
    assert!(kind.is_lexical_file());
    assert!(!kind.is_lexical_symlink());
    assert!(kind.target_is_file());
    assert_eq!(inventory.metadata_stat_count(), 1);
}

#[test]
fn git_untracked_files_still_stat() {
    let dir = saved_ts_source("git-index-classify");
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    // Created after git add so the path stays untracked. Content is not observed.
    std::fs::copy(
        dir.path().join("regular.mts"),
        dir.path().join("untracked.mts"),
    )
    .unwrap();

    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(dir.path());
    let sources = snapshot.source_store_for(dir.path());
    let inventory = sources.inventory();
    assert!(inventory
        .paths()
        .iter()
        .any(|path| path.ends_with("untracked.mts")));
    assert_eq!(inventory.metadata_stat_count(), 1);
}

#[test]
fn git_skip_worktree_missing_files_are_omitted() {
    let dir = saved_ts_source("git-index-classify");
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    crate::test_support::git_skip_worktree(dir.path(), "extra.mts");
    std::fs::remove_file(dir.path().join("extra.mts")).unwrap();

    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(dir.path());
    let sources = snapshot.source_store_for(dir.path());
    let inventory = sources.inventory();
    assert!(
        !inventory
            .paths()
            .iter()
            .any(|path| path.ends_with("extra.mts")),
        "absent skip-worktree files must not stay in the inventory"
    );
    assert_eq!(
        inventory.metadata_stat_count(),
        1,
        "absent skip-worktree files still require a metadata attempt"
    );
}

#[test]
fn git_skip_worktree_present_files_still_stat() {
    let dir = saved_ts_source("git-index-classify");
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    crate::test_support::git_skip_worktree(dir.path(), "extra.mts");

    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(dir.path());
    let sources = snapshot.source_store_for(dir.path());
    let inventory = sources.inventory();
    assert!(inventory
        .paths()
        .iter()
        .any(|path| path.ends_with("extra.mts")));
    assert_eq!(inventory.metadata_stat_count(), 1);
}

#[cfg(unix)]
#[test]
fn git_untracked_stage_shaped_names_keep_the_literal_path() {
    let dir = saved_ts_source("git-index-classify");
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    // The tab in the name is the behavior under test. Copy after git add so
    // the path stays untracked; a saved filename would be staged by git_add_all.
    let spoofed = dir.path().join("100644 abcdef 0\tactual.mts");
    std::fs::copy(dir.path().join("regular.mts"), &spoofed).unwrap();

    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(dir.path());
    let sources = snapshot.source_store_for(dir.path());
    let inventory = sources.inventory();
    assert!(
        inventory.paths().iter().any(|path| path == &spoofed),
        "untracked names that resemble --stage metadata must stay literal"
    );
}

#[test]
fn git_classify_drops_missing_relative_paths() {
    let root = fixture("alpha.ts").parent().unwrap().to_path_buf();
    let (classified, stats) = super::super::classify_git_listed_paths(
        &root,
        vec![
            (PathBuf::from("alpha.ts"), None),
            (PathBuf::from("missing.ts"), None),
            (
                PathBuf::from("missing-link.ts"),
                Some(super::super::GitIndexKind::Symlink),
            ),
        ],
    );
    assert_eq!(classified.len(), 1);
    assert!(classified[0].path.ends_with("alpha.ts"));
    assert_eq!(stats, 3);
}

#[test]
fn git_classify_tracked_regular_missing_paths_skip_metadata() {
    let root = fixture("alpha.ts").parent().unwrap().to_path_buf();
    let (classified, stats) = super::super::classify_git_listed_paths(
        &root,
        vec![(
            PathBuf::from("missing-regular.ts"),
            Some(super::super::GitIndexKind::RegularFile),
        )],
    );
    assert_eq!(classified.len(), 1);
    assert_eq!(stats, 0);
}
