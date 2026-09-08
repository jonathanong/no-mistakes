use super::*;
use std::path::Path;

fn mapping(source: &str) -> Mapping {
    serde_yaml::from_str::<Value>(source)
        .unwrap()
        .as_mapping()
        .unwrap()
        .clone()
}

#[test]
fn globs_accept_string_and_sequence_and_reject_invalid_values() {
    let values = mapping(
        "files: '**/*.ts'\nexcludeFiles: [skip.ts]\ninvalid: [1]\nbad: '['\nscalar: true\nempty: []\nblank: ''\nwhitespace: '   '\ndot: './'\ndotWhitespace: ' ./ '\n",
    );
    assert!(compile_value_globs(&values, "files")
        .unwrap()
        .is_match("src/file.ts"));
    assert!(optional_value_globs(&values, "missing").unwrap().is_none());
    assert!(optional_value_globs(&values, "excludeFiles")
        .unwrap()
        .unwrap()
        .is_match("skip.ts"));
    assert!(compile_value_globs(&values, "invalid").is_none());
    assert!(compile_value_globs(&values, "bad").is_none());
    assert!(compile_value_globs(&values, "scalar").is_none());
    assert!(compile_value_globs(&values, "empty").is_none());
    assert!(compile_value_globs(&values, "blank").is_none());
    assert!(compile_value_globs(&values, "whitespace").is_none());
    assert!(compile_value_globs(&values, "dot").is_none());
    assert!(compile_value_globs(&values, "dotWhitespace").is_none());
    assert!(optional_value_globs(&values, "invalid").is_err());
}

#[test]
fn relative_paths_support_siblings_and_reject_different_roots() {
    assert_eq!(
        relative_path(Path::new("/repo/base"), Path::new("/repo/app/a.ts")),
        Some("../app/a.ts".to_string())
    );
    assert_eq!(
        relative_path(Path::new("/repo/base"), Path::new("/repo/base/a.ts")),
        Some("a.ts".to_string())
    );
    assert_eq!(
        relative_path(Path::new("/repo"), Path::new("relative")),
        None
    );
    assert_eq!(
        relative_path(Path::new("/repo"), Path::new("/repo/../outside")),
        Some("outside".to_string())
    );
}
