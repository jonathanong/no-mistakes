use super::super::mode::ConditionMode;
use super::{source_targets, targets};
use serde_json::json;
use std::path::Path;

fn source(
    exports: serde_json::Value,
    subpath: &str,
    mode: ConditionMode,
) -> Result<Vec<String>, ()> {
    Ok(source_targets(&exports, subpath, mode)?
        .into_iter()
        .map(|target| target.path)
        .collect())
}

#[test]
fn star_subpath_substitutes_the_capture_into_the_target() {
    let found = source(
        json!({ "./*": "./*.mts" }),
        "sub-module",
        ConditionMode::Import,
    )
    .unwrap();
    assert_eq!(found, vec!["./sub-module.mts".to_string()]);
}

#[test]
fn an_exact_export_key_wins_over_a_star_pattern() {
    let found = source(
        json!({ "./*": "./*.mts", "./exact": "./exact.mts" }),
        "exact",
        ConditionMode::Import,
    )
    .unwrap();
    assert_eq!(found, vec!["./exact.mts".to_string()]);
}

#[test]
fn import_and_require_conditions_pick_their_star_target() {
    let exports = json!({ "./*": { "import": "./*.mts", "require": "./*.cts" } });
    assert_eq!(
        source(exports.clone(), "list-files", ConditionMode::Import).unwrap(),
        vec!["./list-files.mts".to_string()]
    );
    assert_eq!(
        source(exports, "list-files", ConditionMode::Require).unwrap(),
        vec!["./list-files.cts".to_string()]
    );
}

#[test]
fn a_longer_star_prefix_wins() {
    let found = source(
        json!({ "./*": "./broad/*.mts", "./sub/*": "./nested/*.mts" }),
        "sub/name",
        ConditionMode::Import,
    )
    .unwrap();
    assert_eq!(found, vec!["./nested/name.mts".to_string()]);
}

#[test]
fn a_pattern_without_a_star_in_the_target_does_not_resolve() {
    assert!(source(
        json!({ "./*": "./src/file.mts" }),
        "file",
        ConditionMode::Import
    )
    .is_err());
    assert!(source(
        json!({ "./*/*": "./*/*.mts" }),
        "a/b",
        ConditionMode::Import
    )
    .is_err());
    assert!(source(json!({ "./*": "./*.mts" }), "../x", ConditionMode::Import).is_err());
}

#[test]
fn declaration_targets_substitute_a_star_pattern() {
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let (typed, found) = targets(
        &json!({ "./*": { "types": "./*.d.ts" } }),
        "name",
        Path::new("."),
        ConditionMode::Import,
        &sources,
    )
    .unwrap();
    assert!(typed);
    assert_eq!(found[0].path, "./name.d.ts");
    let (typed, found) = targets(
        &json!({ "./*": "./*.js" }),
        "list-files",
        Path::new("."),
        ConditionMode::Import,
        &sources,
    )
    .unwrap();
    assert!(!typed);
    assert_eq!(found[0].path, "./list-files.d.ts");
}
