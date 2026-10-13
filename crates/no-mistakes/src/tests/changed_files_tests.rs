use super::*;

#[test]
fn git_name_status_preserves_deleted_and_renamed_paths() {
    let changed =
        parse_git_name_status_z(b"M\0alive.cs\0D\0deleted.cs\0R100\0old-name.cs\0new-name.cs\0")
            .unwrap();

    assert_eq!(
        changed.files,
        vec![
            PathBuf::from("alive.cs"),
            PathBuf::from("deleted.cs"),
            PathBuf::from("new-name.cs"),
            PathBuf::from("old-name.cs"),
        ]
    );
    assert_eq!(
        changed.deleted,
        vec![PathBuf::from("deleted.cs"), PathBuf::from("old-name.cs")]
    );
}

#[test]
fn git_name_status_preserves_both_copy_paths_without_marking_the_source_deleted() {
    let changed = parse_git_name_status_z(b"C100\0source.cs\0copied.cs\0").unwrap();

    assert_eq!(
        changed.files,
        vec![PathBuf::from("copied.cs"), PathBuf::from("source.cs")]
    );
    assert!(changed.deleted.is_empty());
}

#[test]
fn git_name_status_nul_format_preserves_newlines_spaces_unicode_and_leading_dashes() {
    let changed = parse_git_name_status_z(
        "M\0line\nbreak.cs\0M\0space name.cs\0M\0日本語.cs\0M\0-leading.cs\0".as_bytes(),
    )
    .unwrap();

    assert_eq!(
        changed.files,
        vec![
            PathBuf::from("-leading.cs"),
            PathBuf::from("line\nbreak.cs"),
            PathBuf::from("space name.cs"),
            PathBuf::from("日本語.cs"),
        ]
    );
}

#[cfg(unix)]
#[test]
fn manual_symlink_keeps_lexical_inventory_path_and_resolves_analysis_target() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("target.ts"), "export const value = 1;\n").unwrap();
    std::os::unix::fs::symlink("target.ts", root.path().join("alias.ts")).unwrap();
    let canonical_root = root.path().canonicalize().unwrap();

    let (inventory, analysis) =
        resolve_manual_path(Path::new("alias.ts"), &canonical_root).unwrap();

    assert_eq!(inventory, canonical_root.join("alias.ts"));
    assert_eq!(analysis, canonical_root.join("target.ts"));
}

#[test]
fn manual_changed_path_rejects_lexical_and_resolved_root_escape() {
    let root = tempfile::tempdir().unwrap();
    let canonical_root = root.path().canonicalize().unwrap();
    let lexical = resolve_manual_path(Path::new("../outside.ts"), &canonical_root).unwrap_err();
    assert!(lexical.to_string().contains("outside the project root"));

    #[cfg(unix)]
    {
        let outside = tempfile::NamedTempFile::new().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("escape.ts")).unwrap();
        let resolved = resolve_manual_path(Path::new("escape.ts"), &canonical_root).unwrap_err();
        assert!(
            resolved
                .to_string()
                .contains("resolves outside the project root")
        );
    }
}

#[test]
fn refspec_three_dot_splits_base_and_head() {
    let (base, head) = parse_git_diff_refspec("origin/main...HEAD").unwrap();
    assert_eq!(base, "origin/main");
    assert_eq!(head.as_deref(), Some("HEAD"));
}

#[test]
fn refspec_three_dot_with_trailing_dots_defaults_head() {
    let (base, head) = parse_git_diff_refspec("origin/main...").unwrap();
    assert_eq!(base, "origin/main");
    assert_eq!(head, None);
}

#[test]
fn refspec_bare_base_defaults_head() {
    let (base, head) = parse_git_diff_refspec("origin/main").unwrap();
    assert_eq!(base, "origin/main");
    assert_eq!(head, None);
}

#[test]
fn refspec_trims_surrounding_whitespace() {
    let (base, head) = parse_git_diff_refspec("  origin/main ... HEAD  ").unwrap();
    assert_eq!(base, "origin/main");
    assert_eq!(head.as_deref(), Some("HEAD"));
}

#[test]
fn refspec_rejects_two_dot_form() {
    let err = parse_git_diff_refspec("origin/main..HEAD").unwrap_err();
    assert!(
        err.to_string().contains("two-dot"),
        "unexpected error: {err}"
    );
}

#[test]
fn refspec_rejects_empty_string() {
    let err = parse_git_diff_refspec("   ").unwrap_err();
    assert!(
        err.to_string().contains("non-empty"),
        "unexpected error: {err}"
    );
}

#[test]
fn refspec_rejects_missing_base_before_three_dots() {
    let err = parse_git_diff_refspec("...HEAD").unwrap_err();
    assert!(
        err.to_string().contains("missing a base"),
        "unexpected error: {err}"
    );
}

// Regression for a review finding on #587: combined mode (an explicit
// `--diff*` input alongside `--base`/`--head`) used to surface a
// generic `git command failed` message on an invalid ref instead of the
// same stable diagnostic code the primary streaming path reports.
#[test]
fn combined_mode_git_failure_reports_a_stable_diagnostic_code() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    crate::test_support::git_init(root);
    fs::write(root.join("f.txt"), "one\n").unwrap();
    crate::test_support::git_commit_all(root, "base");

    let error = get_git_changed_files(root, "not-a-real-ref", Some("HEAD")).unwrap_err();
    let git_diff_error = error
        .downcast_ref::<crate::tests::git_diff::GitDiffError>()
        .expect("expected a GitDiffError");
    assert_eq!(git_diff_error.code(), "git-merge-base-unavailable");
}
