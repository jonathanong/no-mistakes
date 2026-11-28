#[test]
fn cargo_versions_are_semver_and_git_queries_are_decoded_once() {
    assert!(
        packages_for_diff(PackageManager::Cargo, &fixture("cargo", "bad-version.lock")).is_err()
    );
    assert!(packages_for_diff(
        PackageManager::Cargo,
        &fixture("cargo", "legacy-root-bad-version.lock")
    )
    .is_err());
    let versions =
        packages_for_diff(PackageManager::Cargo, &fixture("cargo", "prerelease.lock")).unwrap();
    expect(
        &versions,
        "sample-pre",
        "1.0.0-alpha.1",
        "",
        ResolutionKind::Workspace,
    );
    expect(
        &versions,
        "sample-build",
        "1.0.0+build.5",
        "",
        ResolutionKind::Workspace,
    );
    let base = packages_for_diff(
        PackageManager::Cargo,
        &fixture("cargo", "git-query-base.lock"),
    )
    .unwrap();
    let head = packages_for_diff(
        PackageManager::Cargo,
        &fixture("cargo", "git-query-head.lock"),
    )
    .unwrap();
    let decoded = "git+https://example.invalid/sample.git?branch=feature/foo#abc123";
    expect(&base, "sample", "1.0.0", decoded, ResolutionKind::Git);
    expect(
        &base,
        "sample-open",
        "1.0.0",
        "git+https://example.invalid/sample.git?branch=feature/foo",
        ResolutionKind::Git,
    );
    expect(
        &base,
        "sample-commit",
        "1.0.0",
        "git+https://example.invalid/sample.git?branch=feature/foo#def456",
        ResolutionKind::Git,
    );
    expect(
        &base,
        "sample-rev",
        "1.0.0",
        "git+https://example.invalid/sample.git?rev=abc123#abc123",
        ResolutionKind::Git,
    );
    expect(
        &base,
        "sample-other",
        "1.0.0",
        "git+https://example.invalid/other.git?branch=feature/foo#abc123",
        ResolutionKind::Git,
    );
    expect(
        &base,
        "sample-sum",
        "1.0.0",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ResolutionKind::Git,
    );
    expect(
        &base,
        "sample-bad",
        "1.0.0",
        "git+https://example.invalid/sample.git?branch=feature%ZZ#abc123",
        ResolutionKind::Git,
    );
    expect(
        &base,
        "sample-utf",
        "1.0.0",
        "git+https://example.invalid/sample.git?branch=%FF#abc123",
        ResolutionKind::Git,
    );
    assert_eq!(base, head);
}

#[test]
fn poetry_hashless_archive_urls_are_fingerprints() {
    let base = packages_for_diff(
        PackageManager::Poetry,
        &fixture("poetry", "hashless-url-base.lock"),
    )
    .unwrap();
    let head = packages_for_diff(
        PackageManager::Poetry,
        &fixture("poetry", "hashless-url-head.lock"),
    )
    .unwrap();
    expect(
        &base,
        "sample",
        "1.0.0",
        "https://example.invalid/a.whl",
        ResolutionKind::Tarball,
    );
    assert_eq!(diff(&base, &head).changed, ["sample"]);
}

#[test]
fn uv_versions_must_be_pep440() {
    assert!(packages_for_diff(PackageManager::Uv, &fixture("uv", "bad-version.lock")).is_err());
    let packages =
        packages_for_diff(PackageManager::Uv, &fixture("uv", "prerelease.lock")).unwrap();
    expect(
        &packages,
        "sample-pre",
        "1!2.0.0a1.post2.dev3+local.1",
        "",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-app",
        "",
        "lib/sample",
        ResolutionKind::Directory,
    );
}
