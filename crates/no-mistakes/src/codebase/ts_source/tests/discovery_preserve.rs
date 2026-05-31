use super::write;
use tempfile::TempDir;

#[test]
fn discover_files_preserving_roots_walks_preserved_skip_dir_subtrees() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "src/main.mts", "");
    write(dir.path(), "fixtures/app/src/lib.rs", "");
    write(dir.path(), "fixtures/other/src/lib.rs", "");

    let files = crate::codebase::ts_source::discover_files_preserving_roots(
        dir.path(),
        &["fixtures".to_string()],
        &[dir.path().join("fixtures/app")],
    );

    assert_eq!(
        files,
        vec![
            dir.path().join("fixtures/app/src/lib.rs"),
            dir.path().join("src/main.mts"),
        ]
    );
}
