use super::*;

#[test]
fn empty_patterns_return_no_files() {
    let files = expand_globs(Path::new("."), &[]).expect("empty globs should succeed");
    assert!(files.is_empty());
}

#[test]
fn skip_dir_matches_generated_and_dependency_directories() {
    for name in [
        ".git",
        ".next",
        ".hidden",
        "node_modules",
        "target",
        "dist",
        "build",
        "coverage",
    ] {
        assert!(is_skip_dir(Path::new(name)), "{name}");
    }
    assert!(!is_skip_dir(Path::new("app")));
}

#[test]
fn dot_directories_excluded_from_glob_expansion() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/react-traits-glob/skip-dot-directories/fixture");
    let files =
        expand_globs(&root, &["**/*.tsx".to_string()]).expect("glob expansion should succeed");
    let names: Vec<&str> = files
        .iter()
        .filter_map(|p| p.file_name()?.to_str())
        .collect();
    assert!(names.contains(&"Button.tsx"), "should find Button.tsx");
    assert!(names.contains(&"Card.tsx"), "should find Card.tsx");
    assert!(
        !names.contains(&"Stale.tsx"),
        "should not find Stale.tsx in .hidden/"
    );
    assert!(
        !names.contains(&"Component.tsx"),
        "should not find Component.tsx in dot directories"
    );
}
