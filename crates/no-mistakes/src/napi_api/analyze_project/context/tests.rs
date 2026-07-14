use super::{canonical_filter_key, same_config_path};
use std::path::Path;

#[test]
fn same_config_path_normalizes_relative_paths_and_preserves_optionality() {
    let root = Path::new("/repo");

    assert!(same_config_path(
        root,
        Some(Path::new("config/../no-mistakes.yml")),
        Some(Path::new("/repo/no-mistakes.yml")),
    ));
    assert!(same_config_path(root, None, None));
    assert!(!same_config_path(
        root,
        Some(Path::new("no-mistakes.yml")),
        None,
    ));
}

#[test]
fn filter_cache_keys_ignore_order_and_duplicates() {
    let left = vec![
        "src/**".to_string(),
        "tests/**".to_string(),
        "src/**".to_string(),
    ];
    let right = vec!["tests/**".to_string(), "src/**".to_string()];
    assert_eq!(
        canonical_filter_key(&left).unwrap(),
        canonical_filter_key(&right).unwrap()
    );
}
