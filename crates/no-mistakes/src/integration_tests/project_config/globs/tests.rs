use super::*;
use std::path::{Path, PathBuf};

#[test]
fn explicit_glob_configs_expand_against_visible_files() {
    let root = Path::new("/repo");
    let visible = [
        PathBuf::from("/repo/packages/app/jest.config.js"),
        PathBuf::from("/repo/packages/app/src/value.test.ts"),
    ]
    .into_iter()
    .collect::<crate::fx::PathSet>();
    let expanded =
        expand_explicit_config_values(root, &["**/jest.config.js".to_string()], &visible);
    assert_eq!(expanded, vec!["packages/app/jest.config.js".to_string()]);
}

#[test]
fn literal_config_paths_keep_configured_order() {
    let expanded = expand_explicit_config_values(
        Path::new("/repo"),
        &[
            "vitest.workspace.json".to_string(),
            "vitest.projects.json".to_string(),
        ],
        &crate::fx::PathSet::default(),
    );
    assert_eq!(
        expanded,
        vec![
            "vitest.workspace.json".to_string(),
            "vitest.projects.json".to_string()
        ]
    );
}

#[test]
fn duplicate_literal_configs_are_kept_once() {
    let expanded = expand_explicit_config_values(
        Path::new("/repo"),
        &["jest.config.js".to_string(), "jest.config.js".to_string()],
        &crate::fx::PathSet::default(),
    );
    assert_eq!(expanded, vec!["jest.config.js".to_string()]);
}

#[test]
fn question_mark_and_character_class_globs_expand() {
    let root = Path::new("/repo");
    let visible = [
        PathBuf::from("/repo/jest.config.js"),
        PathBuf::from("/repo/jest.config.ts"),
    ]
    .into_iter()
    .collect::<crate::fx::PathSet>();
    let question = expand_explicit_config_values(root, &["jest.config.j?".to_string()], &visible);
    assert_eq!(question, vec!["jest.config.js".to_string()]);
    let classed = expand_explicit_config_values(root, &["jest.config.[jt]s".to_string()], &visible);
    assert_eq!(
        classed,
        vec!["jest.config.js".to_string(), "jest.config.ts".to_string()]
    );
}

#[test]
fn malformed_glob_is_skipped() {
    let expanded = expand_explicit_config_values(
        Path::new("/repo"),
        &["jest.config.[js".to_string(), "jest.config.js".to_string()],
        &[PathBuf::from("/repo/jest.config.js")]
            .into_iter()
            .collect::<crate::fx::PathSet>(),
    );
    assert_eq!(expanded, vec!["jest.config.js".to_string()]);
}

#[test]
fn unmatched_glob_adds_no_values() {
    let expanded = expand_explicit_config_values(
        Path::new("/repo"),
        &["**/missing.config.js".to_string()],
        &[PathBuf::from("/repo/jest.config.js")]
            .into_iter()
            .collect::<crate::fx::PathSet>(),
    );
    assert!(expanded.is_empty());
}
