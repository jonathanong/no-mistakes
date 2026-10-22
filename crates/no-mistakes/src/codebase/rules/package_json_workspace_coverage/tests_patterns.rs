use super::super::check_with_files;
use super::super::patterns::unmatched_patterns;
use super::{config, files_named, fixture_root};

#[test]
fn positive_patterns_match_alone_and_invalid_globs_fail() {
    let missing = unmatched_patterns(
        &[
            "packages/*".to_string(),
            "apps/*".to_string(),
            "!packages/hidden".to_string(),
        ],
        &["packages/hidden".to_string()],
    )
    .unwrap();
    assert_eq!(missing, vec!["apps/*".to_string()]);
    assert!(unmatched_patterns(&["[".to_string()], &[]).is_err());
}

#[test]
fn unmatched_workspace_entries_ignore_negated_patterns() {
    let root = fixture_root("unmatched-pattern");
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages, apps]\n"),
        &files_named(
            &root,
            &[
                "pnpm-workspace.yaml",
                "packages/hidden/package.json",
                "packages/group/app/package.json",
            ],
        ),
    )
    .unwrap();
    let body = findings
        .iter()
        .map(|finding| finding.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        body.contains("workspace pattern \"apps/*\" does not correspond to a package directory"),
        "{body}"
    );
    assert!(!body.contains("workspace pattern \"packages/*\""), "{body}");
    assert!(!body.contains("workspace pattern \"!"), "{body}");
    assert!(body.contains("packages/hidden/package.json"), "{body}");
    assert!(body.contains("packages/group/app/package.json"), "{body}");
    assert_eq!(findings[0].file, "packages/group/app/package.json");
}

#[test]
fn unmatched_package_json_workspaces_attach_to_the_root_manifest() {
    let root = fixture_root("unmatched-package-json");
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages]\n"),
        &files_named(&root, &["package.json", "packages/hidden/package.json"]),
    )
    .unwrap();
    assert!(
        findings.iter().any(|finding| {
            finding.file == "package.json"
                && finding.message.contains("workspace pattern \"apps/*\"")
        }),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("workspace pattern \"packages/*\"")),
        "{findings:?}"
    );
}

#[test]
fn included_package_globs_are_relative_to_the_included_file() {
    let root = fixture_root("included-packages");
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages, services]\n"),
        &files_named(
            &root,
            &[
                "pnpm-workspace.yaml",
                "pnpm/packages.yaml",
                "packages/api/package.json",
                "packages/group/app/package.json",
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
        vec!["packages/group/app/package.json", "services/package.json"]
    );
}
