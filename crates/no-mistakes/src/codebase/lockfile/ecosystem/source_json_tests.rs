#[test]
fn pipfile_git_and_path_omit_version() {
    let packages = packages_for_diff(
        PackageManager::Pipfile,
        &fixture("pipfile", "versionless.lock"),
    )
    .unwrap();
    assert_eq!(packages.len(), 2);
    expect(&packages, "sample-git", "", "abc123", ResolutionKind::Git);
    expect(&packages, "sample-path", "", ".", ResolutionKind::Directory);
}

#[test]
fn pipfile_direct_sources_without_versions_keep_registry_packages() {
    let packages = packages_for_diff(
        PackageManager::Pipfile,
        &fixture("pipfile", "direct-sources.lock"),
    )
    .unwrap();
    for (name, source) in [
        (
            "sample-file",
            "https://packages.example.invalid/archive.whl",
        ),
        (
            "sample-url",
            "https://packages.example.invalid/package.tar.gz",
        ),
        ("sample-hg", "https://vcs.example.invalid/hg"),
        ("sample-svn", "https://vcs.example.invalid/svn"),
        ("sample-bzr", "https://vcs.example.invalid/bzr"),
    ] {
        let kind = if ["sample-file", "sample-url"].contains(&name) {
            ResolutionKind::Tarball
        } else {
            ResolutionKind::Other
        };
        expect(&packages, name, "", source, kind);
    }
    expect(
        &packages,
        "sample-registry",
        "1.0.0",
        "sha256:abcd",
        ResolutionKind::Registry,
    );
}

#[test]
fn composer_null_development_section_preserves_production_packages() {
    let packages = packages_for_diff(
        PackageManager::Composer,
        &fixture("composer", "no-dev.lock"),
    )
    .unwrap();
    assert_eq!(packages.len(), 1);
    expect(
        &packages,
        "example/sample",
        "1.2.3",
        "",
        ResolutionKind::Registry,
    );
}

#[test]
fn pipfile_custom_category_changes_are_reported() {
    let base = packages_for_diff(
        PackageManager::Pipfile,
        &fixture("pipfile", "custom-base.lock"),
    )
    .unwrap();
    let head = packages_for_diff(
        PackageManager::Pipfile,
        &fixture("pipfile", "custom-head.lock"),
    )
    .unwrap();
    assert_eq!(diff(&base, &head).changed, ["sample-docs"]);
}

#[test]
fn composer_diffs_path_and_checksum_without_references() {
    let base = packages_for_diff(
        PackageManager::Composer,
        &fixture("composer", "fingerprints-base.lock"),
    )
    .unwrap();
    let head = packages_for_diff(
        PackageManager::Composer,
        &fixture("composer", "fingerprints-head.lock"),
    )
    .unwrap();
    assert_eq!(
        diff(&base, &head).changed,
        ["example/checked", "example/path"]
    );
}

#[test]
fn pipfile_non_git_vcs_revision_changes_are_reported() {
    let base = packages_for_diff(
        PackageManager::Pipfile,
        &fixture("pipfile", "revision-base.lock"),
    )
    .unwrap();
    let head = packages_for_diff(
        PackageManager::Pipfile,
        &fixture("pipfile", "revision-head.lock"),
    )
    .unwrap();
    assert_eq!(diff(&base, &head).changed, ["sample-checkout"]);
}

#[test]
fn json_lockfiles_reject_invalid_optional_fields() {
    for name in ["invalid-dist.lock", "invalid-source.lock"] {
        assert!(packages_for_diff(PackageManager::Composer, &fixture("composer", name)).is_err());
    }
    let packages = packages_for_diff(
        PackageManager::Composer,
        &fixture("composer", "null-dist.lock"),
    )
    .unwrap();
    expect(
        &packages,
        "example/sample",
        "1.0.0",
        "abc123",
        ResolutionKind::Git,
    );
    assert!(packages_for_diff(
        PackageManager::Pipfile,
        &fixture("pipfile", "invalid-version.lock")
    )
    .is_err());
}

#[test]
fn pipfile_hashes_must_be_strings_and_noneditable_packages_use_them() {
    for name in ["invalid-hashes.lock", "invalid-hash-entry.lock"] {
        assert!(packages_for_diff(PackageManager::Pipfile, &fixture("pipfile", name)).is_err());
    }
    let packages = packages_for_diff(
        PackageManager::Pipfile,
        &fixture("pipfile", "not-editable.lock"),
    )
    .unwrap();
    expect(
        &packages,
        "sample",
        "1.0.0",
        "sha256:sample",
        ResolutionKind::Registry,
    );
}
