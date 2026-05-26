use super::*;

#[test]
fn maps_adjacent_and_rooted_manual_mocks_to_targets() {
    let root = PathBuf::from("/repo");
    let adjacent = root.join("src/__mocks__/manual.mts");
    let targets = mocked_targets(&root, &adjacent);
    assert!(targets.contains(&root.join("src/manual.mts")));
    assert!(!targets.contains(&root.join("manual.mts")));

    let rooted = root.join("__mocks__/src/manual.mts");
    let targets = mocked_targets(&root, &rooted);
    assert!(targets.contains(&root.join("src/manual.mts")));
    assert!(targets.contains(&PathBuf::from("src/manual")));
}

#[test]
fn root_manual_mock_maps_unresolved_module_specifier() {
    let root = PathBuf::from("/repo");
    let mock = root.join("__mocks__/external-lib.js");
    let targets = mocked_targets(&root, &mock);
    assert!(targets.contains(&PathBuf::from("external-lib")));
}

#[test]
fn discover_respects_skip_directories() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports/fixture"),
    );
    let skipped = root.join("skipped").join("ignored.mts");
    let mocks = discover(&root, &["skipped".to_string()]);
    assert!(!mocks.contains(&skipped));
}
