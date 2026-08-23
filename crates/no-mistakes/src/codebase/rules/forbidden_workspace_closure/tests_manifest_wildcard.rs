use super::*;

#[test]
fn wildcard_version_range_extends_manifest_workspace_closure() {
    let root = fixture_root("manifest-wildcard-range");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
            "packages/secret/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/domain -> @acme/secret")
    );
}
