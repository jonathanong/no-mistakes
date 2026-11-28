//! Saved-fixture tests and snapshots for LCOV path normalization and security (#4800 Checklist §C).

mod common;

use common::{pretty_json, read_fixture_bytes};
use no_mistakes_coverage::{
    normalize_path, parse_lcov_bytes, NormalizeOptions, ParseDiagnosticKind, ParseOptions,
    PathNormalizeError,
};

#[test]
fn normalizes_posix_root_prefixes_relative_dots_and_file_urls() {
    let options = ParseOptions::with_normalize(
        NormalizeOptions::default()
            .with_root("/home/runner/work/repo/repo")
            .with_strip_prefix("/custom/strip/")
            .with_strip_ci_work_prefix(false),
    );
    let parsed = parse_lcov_bytes(
        &read_fixture_bytes("path_normalization_posix_root.lcov"),
        &options,
    );
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.report.files["web/lib/client.mts"].lines[&1], 6);
    assert_eq!(parsed.report.files["src/my file.ts"].lines[&1], 4);
    assert_eq!(parsed.report.files["src/localhost.ts"].lines[&1], 5);
    assert_eq!(parsed.report.files["src/triple_slash.ts"].lines[&1], 6);
    assert!(parsed
        .report
        .files
        .contains_key("/home/runner/work/repo/repo-other/src/sibling.ts"));
    insta::assert_snapshot!("path_normalization_posix_root", pretty_json(&parsed));
}

#[test]
fn normalizes_windows_drive_case_backslashes_file_urls_and_unc_paths() {
    let options = ParseOptions::with_normalize(
        NormalizeOptions::default()
            .with_root(r"C:\repo")
            .with_strip_prefix(r"\\fileserver\share\repo"),
    );
    let parsed = parse_lcov_bytes(
        &read_fixture_bytes("path_normalization_windows.lcov"),
        &options,
    );
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.report.files["backend/services/foo.mts"].lines[&1], 6);
    assert_eq!(
        parsed.report.files["backend/services/url_win.mts"].lines[&1],
        4
    );
    assert_eq!(parsed.report.files["backend/unc.mts"].lines[&1], 5);
    insta::assert_snapshot!("path_normalization_windows", pretty_json(&parsed));
}

#[test]
fn auto_strips_ci_work_prefixes_and_preserves_other_absolute_paths() {
    let parsed = parse_lcov_bytes(
        &read_fixture_bytes("path_normalization_ci_work.lcov"),
        &ParseOptions::default(),
    );
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.report.files["src/api.mts"].lines[&1], 1);
    assert_eq!(parsed.report.files["src/index.mts"].lines[&1], 2);
    assert_eq!(parsed.report.files["src/win_api.mts"].lines[&1], 3);
    assert_eq!(
        parsed.report.files["/usr/local/lib/external.mts"].lines[&1],
        4
    );
    insta::assert_snapshot!("path_normalization_ci_work", pretty_json(&parsed));
}

#[test]
fn rejects_unsafe_and_outside_root_paths_scoped_to_their_record() {
    let options = ParseOptions::with_normalize(
        NormalizeOptions::default()
            .with_root("/repo")
            .with_strip_ci_work_prefix(false)
            .with_reject_outside_root(true),
    );
    let parsed = parse_lcov_bytes(&read_fixture_bytes("path_security.lcov"), &options);
    assert_eq!(parsed.report.len(), 1);
    assert_eq!(parsed.report.files["src/safe.ts"].lines[&10], 2);
    assert_eq!(parsed.diagnostics.len(), 9);
    assert!(parsed
        .diagnostics
        .iter()
        .all(|d| d.kind == ParseDiagnosticKind::InvalidPath));
    insta::assert_snapshot!("path_security", pretty_json(&parsed));
}

#[test]
fn covers_direct_path_normalization_variants_and_errors() {
    let posix_root = NormalizeOptions::default()
        .with_root("/")
        .with_reject_outside_root(true);
    assert_eq!(
        normalize_path("/src/root_child.ts", &posix_root),
        Ok("src/root_child.ts".to_string())
    );

    let win_drive_root = NormalizeOptions::default()
        .with_root(r"C:\")
        .with_reject_outside_root(true);
    assert_eq!(
        normalize_path(r"c:\dir\file.ts", &win_drive_root),
        Ok("dir/file.ts".to_string())
    );
    assert_eq!(
        normalize_path("/etc/passwd", &win_drive_root),
        Err(PathNormalizeError::ForeignAbsolutePath)
    );

    let default_opts = NormalizeOptions::default();
    assert_eq!(
        normalize_path("", &default_opts),
        Err(PathNormalizeError::Empty)
    );
    assert_eq!(
        normalize_path("a\0b", &default_opts),
        Err(PathNormalizeError::ContainsNul)
    );
    assert_eq!(
        normalize_path("../a.ts", &default_opts),
        Err(PathNormalizeError::EscapesRoot)
    );
    assert_eq!(
        normalize_path("file:///%2A%2f%zz/a.ts", &default_opts),
        Ok("/*/%zz/a.ts".to_string())
    );
    assert_eq!(
        normalize_path(r"\\server", &default_opts),
        Err(PathNormalizeError::Empty)
    );
    assert_eq!(
        normalize_path(r"\\server\", &default_opts),
        Err(PathNormalizeError::Empty)
    );
    assert_eq!(
        normalize_path(r"\\server\share", &default_opts),
        Ok("//server/share".to_string())
    );
    assert_eq!(normalize_path("C:", &default_opts), Ok("C:/".to_string()));

    let strict_posix = NormalizeOptions::default()
        .with_root("/repo")
        .with_strip_ci_work_prefix(false)
        .with_reject_outside_root(true);
    assert_eq!(
        normalize_path("/other/file.ts", &strict_posix),
        Err(PathNormalizeError::OutsideRoot)
    );

    for err in [
        PathNormalizeError::Empty,
        PathNormalizeError::ContainsNul,
        PathNormalizeError::EscapesRoot,
        PathNormalizeError::ForeignAbsolutePath,
        PathNormalizeError::OutsideRoot,
    ] {
        assert!(!err.to_string().is_empty());
    }
}
