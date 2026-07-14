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
}
