use super::{ClassifiedPath, FileClassification, FileId, FileInventory};
use std::path::PathBuf;
use std::time::Instant;

fn fixture(path: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/analysis-dataset/source-store")
            .join(path),
    )
}

#[test]
fn identities_are_sorted_deduplicated_and_lexically_normalized() {
    let alpha = fixture("alpha.ts");
    let beta = fixture("beta.ts");
    let inventory = FileInventory::from_paths(&[
        beta.clone(),
        alpha.parent().unwrap().join("nested/../alpha.ts"),
        alpha.clone(),
    ]);

    assert_eq!(inventory.len(), 2);
    assert!(!inventory.is_empty());
    assert_eq!(inventory.paths().as_slice(), [alpha.clone(), beta.clone()]);
    assert_eq!(inventory.id_for_path(&alpha).unwrap().index(), 0);
    assert_eq!(inventory.id_for_path(&beta).unwrap().index(), 1);
    assert_eq!(
        inventory.path(inventory.id_for_path(&alpha).unwrap()),
        Some(alpha.as_path())
    );
    assert_eq!(inventory.id_for_path(&fixture("missing.ts")), None);
    assert_eq!(inventory.path(FileId(u32::MAX)), None);
}

#[test]
fn identity_assignment_is_independent_of_candidate_order() {
    let alpha = fixture("alpha.ts");
    let beta = fixture("beta.ts");
    let forward = FileInventory::from_paths(&[alpha.clone(), beta.clone()]);
    let reverse = FileInventory::from_paths(&[beta.clone(), alpha.clone()]);

    assert_eq!(forward.paths(), reverse.paths());
    assert_eq!(forward.id_for_path(&alpha), reverse.id_for_path(&alpha));
    assert_eq!(forward.id_for_path(&beta), reverse.id_for_path(&beta));
}

#[test]
fn empty_inventory_has_no_paths_or_ids() {
    let inventory = FileInventory::from_paths(&[]);

    assert!(inventory.is_empty());
    assert_eq!(inventory.len(), 0);
    assert!(inventory.paths().is_empty());
    assert_eq!(inventory.id_for_path(&fixture("alpha.ts")), None);
}

#[test]
fn logical_symlink_and_target_paths_remain_distinct() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/scan-config/symlinked-default-playwright/fixture");
    let symlink = root.join("playwright.config.ts");
    let target = root.join("configs/shared.playwright.config.ts");
    let inventory = FileInventory::from_paths(&[symlink.clone(), target.clone()]);

    // Do not canonicalize these paths: import resolution is intentionally
    // allowed to distinguish the configured symlink from its target.
    assert_ne!(
        inventory.id_for_path(&symlink),
        inventory.id_for_path(&target)
    );
    assert_eq!(inventory.len(), 2);
    let symlink_kind = inventory.classification_for_path(&symlink).unwrap();
    assert!(!symlink_kind.is_lexical_file());
    assert!(symlink_kind.is_lexical_symlink());
    assert!(symlink_kind.target_is_file());
    assert!(symlink_kind.is_lexical_file() || symlink_kind.is_lexical_symlink());
    let target_kind = inventory.classification_for_path(&target).unwrap();
    assert!(target_kind.is_lexical_file());
    assert!(!target_kind.is_lexical_symlink());
    assert!(target_kind.target_is_file());
    assert!(target_kind.is_lexical_file() || target_kind.is_lexical_symlink());
}

#[test]
fn classified_discovery_does_not_restat_inventory_paths() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/scan-config/symlinked-default-playwright/fixture");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let classified = sources.inventory();
    assert!(
        classified.len() >= 2,
        "symlink fixture must contribute visible paths"
    );
    let symlink = crate::codebase::ts_resolver::normalize_path(&root.join("playwright.config.ts"));
    let symlink_kind = classified.classification_for_path(&symlink).unwrap();
    assert!(symlink_kind.is_lexical_symlink());
    assert!(
        classified.metadata_stat_count() < classified.len(),
        "git index modes must skip stats for tracked regular files"
    );

    let restated = FileInventory::from_paths(classified.paths().as_slice());
    assert_eq!(restated.paths(), classified.paths());
    assert!(
        restated.metadata_stat_count() >= classified.len(),
        "from_paths must stat each candidate; classified reuse must not"
    );
    assert!(restated
        .classification_for_path(&symlink)
        .unwrap()
        .is_lexical_symlink());
}

#[test]
fn non_file_entries_have_no_file_classification() {
    let directory = fixture("alpha.ts").parent().unwrap().to_path_buf();
    let inventory = FileInventory::from_paths(std::slice::from_ref(&directory));
    let classification = inventory.classification_for_path(&directory).unwrap();

    assert!(!classification.is_lexical_file());
    assert!(!classification.is_lexical_symlink());
    assert!(!classification.target_is_file());
    assert!(!classification.is_lexical_file() && !classification.is_lexical_symlink());
    assert!(inventory.non_file_path_entry_paths().is_empty());
    assert!(inventory.target_file_paths().is_empty());
}

#[cfg(unix)]
#[test]
fn directory_target_symlink_is_a_path_entry_not_a_target_file() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/symlink-workspace");
    let symlink = crate::codebase::ts_resolver::normalize_path(&root.join("link"));
    let inventory = FileInventory::from_paths(std::slice::from_ref(&symlink));
    let classification = inventory.classification_for_path(&symlink).unwrap();

    assert!(!classification.is_lexical_file());
    assert!(classification.is_lexical_symlink());
    assert!(!classification.target_is_file());
    assert!(classification.is_lexical_file() || classification.is_lexical_symlink());
    assert_eq!(inventory.non_file_path_entry_paths(), vec![symlink.clone()]);
    assert!(inventory.target_file_paths().is_empty());
}

#[cfg(unix)]
#[test]
fn broken_symlink_is_a_path_entry_not_a_target_file() {
    let broken = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/tests-impact/fixture/broken.test.mts"),
    );
    let inventory = FileInventory::from_paths(std::slice::from_ref(&broken));
    let classification = inventory.classification_for_path(&broken).unwrap();

    assert!(classification.is_lexical_symlink());
    assert!(!classification.target_is_file());
    assert!(classification.is_lexical_file() || classification.is_lexical_symlink());
    assert_eq!(inventory.non_file_path_entry_paths(), vec![broken.clone()]);
    assert!(inventory.target_file_paths().is_empty());
}

fn serial_inventory(paths: &[PathBuf]) -> FileInventory {
    let entries = paths
        .iter()
        .take_while(|_| crate::invocation::check_timeout().is_ok())
        .map(|path| {
            let path = crate::codebase::ts_source::normalize_discovery_path(path);
            let classification = std::fs::symlink_metadata(&path)
                .ok()
                .map_or_else(FileClassification::default, |metadata| {
                    FileClassification::from_file_type(&path, metadata.file_type())
                });
            ClassifiedPath {
                path,
                classification,
            }
        })
        .collect();
    FileInventory::from_classified_paths_counted(entries, 0)
}

fn assert_same_inventory(left: &FileInventory, right: &FileInventory) {
    assert_eq!(left.paths(), right.paths());
    for path in left.paths().iter() {
        let left_kind = left.classification_for_path(path).unwrap();
        let right_kind = right.classification_for_path(path).unwrap();
        assert_eq!(left_kind.is_lexical_file(), right_kind.is_lexical_file());
        assert_eq!(
            left_kind.is_lexical_symlink(),
            right_kind.is_lexical_symlink()
        );
        assert_eq!(left_kind.target_is_file(), right_kind.target_is_file());
    }
}

#[test]
fn parallel_inventory_matches_serial_classification() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/scan-config/symlinked-default-playwright/fixture");
    let missing = fixture("missing.ts");
    let paths = vec![
        fixture("beta.ts"),
        fixture("alpha.ts"),
        root.join("playwright.config.ts"),
        root.join("configs/shared.playwright.config.ts"),
        fixture("alpha.ts").parent().unwrap().to_path_buf(),
        missing,
    ];

    assert_same_inventory(
        &serial_inventory(&paths),
        &FileInventory::from_paths(&paths),
    );
}

#[test]
fn parallel_inventory_is_stable_across_runs() {
    let paths = vec![fixture("alpha.ts"), fixture("beta.ts")];
    assert_same_inventory(
        &FileInventory::from_paths(&paths),
        &FileInventory::from_paths(&paths),
    );
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
    std::fs::write(&link, "export {}\n").unwrap();

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
    std::fs::write(dir.path().join("untracked.mts"), "export {}\n").unwrap();

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
    let spoofed = dir.path().join("100644 abcdef 0\tactual.mts");
    std::fs::write(&spoofed, "export {}\n").unwrap();

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
    let (classified, stats) = super::classify_git_listed_paths(
        &root,
        vec![
            (PathBuf::from("alpha.ts"), None),
            (PathBuf::from("missing.ts"), None),
            (
                PathBuf::from("missing-link.ts"),
                Some(super::GitIndexKind::Symlink),
            ),
        ],
    );
    assert_eq!(classified.len(), 1);
    assert!(classified[0].path.ends_with("alpha.ts"));
    assert_eq!(stats, 3);
}

#[test]
#[ignore = "manual discovery-classify benchmark"]
fn time_parallel_vs_serial_classification() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/large-graph-monorepo/fixture");
    let mut paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    paths.extend(crate::codebase::ts_source::discover_visible_paths(&repo));
    paths.sort();
    paths.dedup();
    let _ = serial_inventory(&paths);
    let _ = FileInventory::from_paths(&paths);

    let started = Instant::now();
    let serial = serial_inventory(&paths);
    let serial_elapsed = started.elapsed();
    let started = Instant::now();
    let parallel = FileInventory::from_paths(&paths);
    let parallel_elapsed = started.elapsed();
    eprintln!(
        "discovery classify paths={} serial={:?} parallel={:?}",
        paths.len(),
        serial_elapsed,
        parallel_elapsed
    );
    assert_same_inventory(&serial, &parallel);
}
