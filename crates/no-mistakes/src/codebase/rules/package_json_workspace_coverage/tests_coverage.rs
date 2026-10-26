use super::super::check_with_files;
use super::{config, files_named, fixture_root};

#[test]
fn duplicate_package_settings_keep_the_root_coverage_finding() {
    let root = fixture_root("duplicate-packages");
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages, services]\n"),
        &files_named(
            &root,
            &[
                "pnpm-workspace.yaml",
                "pnpm/packages.yaml",
                "packages/api/package.json",
                "services/package.json",
            ],
        ),
    )
    .unwrap();
    let files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    assert_eq!(
        files,
        vec!["pnpm-workspace.yaml", "services/package.json"],
        "{findings:?}"
    );
    assert!(findings.iter().any(|finding| {
        finding.file == "pnpm-workspace.yaml"
            && finding
                .message
                .contains("setting \"packages\" is claimed by more than one pnpm source")
    }));
    assert!(findings.iter().any(|finding| {
        finding.file == "services/package.json"
            && finding.message.contains("package directory is not covered")
    }));
}

#[test]
fn narrowed_scope_does_not_guess_coverage_for_duplicate_packages() {
    let root = fixture_root("duplicate-packages");
    let mut narrowed = config("packageRoots: [packages, services]\n");
    narrowed.rules[0].include = vec!["packages/**".to_string()];
    let findings = check_with_files(
        &root,
        &narrowed,
        &files_named(
            &root,
            &[
                "pnpm-workspace.yaml",
                "pnpm/packages.yaml",
                "packages/api/package.json",
                "services/package.json",
            ],
        ),
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn delegated_omitted_packages_do_not_fall_back_to_star() {
    let root = fixture_root("delegated-omitted");
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages, services]\n"),
        &files_named(
            &root,
            &[
                "pnpm-workspace.yaml",
                "pnpm/packages.yaml",
                "packages/api/package.json",
                "services/package.json",
            ],
        ),
    )
    .unwrap();
    let files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    assert_eq!(
        files,
        vec!["packages/api/package.json", "services/package.json"]
    );
}

#[test]
fn unreadable_package_include_is_a_finding_and_does_not_hide_packages() {
    let root = fixture_root("unreadable-packages");
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages, services]\n"),
        &files_named(
            &root,
            &[
                "pnpm-workspace.yaml",
                "packages/api/package.json",
                "services/package.json",
            ],
        ),
    )
    .unwrap();
    let body = format!("{findings:?}");
    assert!(body.contains("could not be read"), "{body}");
    assert!(body.contains("packages/api/package.json"), "{body}");
    assert!(body.contains("services/package.json"), "{body}");
}

#[test]
fn included_scope_does_not_mark_an_existing_outside_package_unmatched() {
    let root = fixture_root("scoped-app");
    let mut narrowed = config("packageRoots: [packages]\n");
    narrowed.rules[0].include = vec!["packages/**".to_string(), "pnpm-workspace.yaml".to_string()];
    let present = files_named(
        &root,
        &[
            "pnpm-workspace.yaml",
            "packages/api/package.json",
            "apps/web/package.json",
        ],
    );
    let findings = check_with_files(&root, &narrowed, &present).unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("apps/*")),
        "{findings:?}"
    );
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
fn excluded_workspace_file_does_not_report_unmatched_patterns() {
    let root = fixture_root("scoped-app");
    let mut narrowed = config("packageRoots: [packages]\n");
    narrowed.rules[0].include = vec!["packages/**".to_string()];
    let files = files_named(&root, &["pnpm-workspace.yaml", "packages/api/package.json"]);
    let findings = check_with_files(&root, &narrowed, &files).unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("apps/*") && finding.file != "package.json"),
        "{findings:?}"
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
