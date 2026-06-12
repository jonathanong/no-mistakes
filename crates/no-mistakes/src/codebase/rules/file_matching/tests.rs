use super::*;

#[test]
fn matching_files_normalizes_relative_globs() {
    let root = Path::new("/repo");
    let files = vec![
        PathBuf::from("/repo/config/app.yml"),
        PathBuf::from("/repo/config/other.yml"),
    ];

    let matched = matching_files(root, &["./config/app.yml".to_string()], &files).unwrap();

    assert_eq!(matched, vec![PathBuf::from("/repo/config/app.yml")]);
}
