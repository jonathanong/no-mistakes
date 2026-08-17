use super::{FileId, FileInventory};
use std::path::PathBuf;

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
    assert_eq!(
        classified.metadata_stat_count(),
        0,
        "snapshot inventory must reuse discovery classifications"
    );

    let restated = FileInventory::from_paths(classified.paths().as_slice());
    assert_eq!(restated.paths(), classified.paths());
    assert!(
        restated.metadata_stat_count() >= classified.len(),
        "from_paths must stat each candidate; classified reuse must not"
    );
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
