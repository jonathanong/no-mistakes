use super::*;

#[test]
fn relative_paths_require_compatible_roots_and_compare_windows_case_insensitively() {
    let cases = [
        ("C:/a", "c:/A/b", Some("b")),
        ("//server/share/a", "//SERVER/SHARE/A/b", Some("b")),
        ("C:/a", "D:/a/b", None),
        ("/a", "relative/a", None),
    ];

    for (root, path, expected) in cases {
        assert_eq!(
            lexical_relative_slash_path(Path::new(root), Path::new(path)).as_deref(),
            expected,
            "root={root:?}, path={path:?}"
        );
    }
}

#[test]
fn rendering_covers_each_prefix_and_parent_normalization_policy() {
    let cases = [
        ("../../a", "../../a"),
        ("/../../a", "/a"),
        ("C:/a/../b", "C:/b"),
        ("//server/share/a/../b", "//server/share/b"),
    ];

    for (path, expected) in cases {
        assert_eq!(lexical_normalized_slash_path(Path::new(path)), expected);
    }
}
