use super::super::check_with_files;
use super::{config, files_named, fixture_root};

// Finding cases moved to cli_pkgcfg_snapshot_package_json_workspace_coverage.

#[test]
fn absent_outside_package_is_reported_unmatched() {
    // apps/web/package.json exists on disk. Omitting it from the explicit list
    // is what makes apps/* unmatched; check discovery cannot drop that file.
    let root = fixture_root("scoped-app");
    let mut narrowed = config("packageRoots: [packages]\n");
    narrowed.rules[0].include = vec!["packages/**".to_string(), "pnpm-workspace.yaml".to_string()];
    let without_app = files_named(&root, &["pnpm-workspace.yaml", "packages/api/package.json"]);
    let missing = check_with_files(&root, &narrowed, &without_app).unwrap();
    assert!(
        missing
            .iter()
            .any(|finding| finding.message.contains("apps/*")),
        "{missing:?}"
    );
}

#[test]
fn invalid_workspace_globs_fail_the_check() {
    let root = fixture_root("invalid-glob");
    let error = check_with_files(
        &root,
        &config("packageRoots: [packages]\n"),
        &files_named(&root, &["pnpm-workspace.yaml", "packages/api/package.json"]),
    )
    .unwrap_err();
    assert!(!error.to_string().is_empty());
}

#[test]
fn invalid_packages_field_fails_the_check() {
    let root = fixture_root("invalid-packages");
    let error = check_with_files(
        &root,
        &config("packageRoots: [packages]\n"),
        &files_named(&root, &["pnpm-workspace.yaml", "packages/api/package.json"]),
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("packages field must be a list of strings"));
}
