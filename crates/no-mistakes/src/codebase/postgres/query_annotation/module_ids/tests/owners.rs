use super::super::ModuleIds;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[test]
fn shared_owner_reuses_only_exact_raw_spelling_without_retaining_the_catalog() {
    let original = PathBuf::from("folder/./file.mts");
    let ids = ModuleIds::new([original.clone()]);
    let first = ids.shared_path(&original);
    assert!(Arc::ptr_eq(&first, &ids.shared_path(&original)));
    let normalized = Path::new("folder/file.mts");
    assert_eq!(original, normalized);
    let alternate = ids.shared_path(normalized);
    assert_eq!(alternate.as_os_str(), normalized.as_os_str());
    assert!(!Arc::ptr_eq(&first, &alternate));
    let missing = ids.shared_path(Path::new("missing.mts"));
    assert_eq!(missing.as_os_str(), "missing.mts");
    drop(ids);
    assert_eq!(first.as_os_str(), original.as_os_str());
    assert_eq!(alternate.as_os_str(), normalized.as_os_str());
}
