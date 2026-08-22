use super::path_ends_with_module;
use std::path::Path;

#[test]
fn wrapper_suffix_matches_path_component_with_extension() {
    assert!(path_ends_with_module(
        Path::new("/repo/src/web/app.ts"),
        "web/app"
    ));
}

#[test]
fn wrapper_suffix_does_not_match_inside_a_directory_name() {
    assert!(!path_ends_with_module(
        Path::new("/repo/src/not-web/app.ts"),
        "web/app"
    ));
}
