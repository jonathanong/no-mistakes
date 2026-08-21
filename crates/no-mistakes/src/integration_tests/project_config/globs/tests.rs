use super::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[test]
fn explicit_glob_configs_expand_against_visible_files() {
    let root = Path::new("/repo");
    let visible = HashSet::from([
        PathBuf::from("/repo/packages/app/jest.config.js"),
        PathBuf::from("/repo/packages/app/src/value.test.ts"),
    ]);
    let expanded =
        expand_explicit_config_values(root, &["**/jest.config.js".to_string()], &visible);
    assert_eq!(expanded, vec!["packages/app/jest.config.js".to_string()]);
}

#[test]
fn literal_config_paths_are_kept() {
    let root = Path::new("/repo");
    let expanded =
        expand_explicit_config_values(root, &["jest.config.js".to_string()], &HashSet::new());
    assert_eq!(expanded, vec!["jest.config.js".to_string()]);
}
