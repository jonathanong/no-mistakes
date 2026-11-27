#[test]
fn cargo_reads_legacy_root_package() {
    let packages =
        packages_for_diff(PackageManager::Cargo, &fixture("cargo", "legacy-root.lock")).unwrap();
    expect(&packages, "sample", "1.0.0", "", ResolutionKind::Workspace);
}

#[test]
fn poetry_selects_the_matching_legacy_archive_hash() {
    let base = packages_for_diff(
        PackageManager::Poetry,
        &fixture("poetry", "legacy-archive-base.lock"),
    )
    .unwrap();
    let head = packages_for_diff(
        PackageManager::Poetry,
        &fixture("poetry", "legacy-archive-head.lock"),
    )
    .unwrap();
    expect(
        &base,
        "sample",
        "1.0.0",
        "sha256:first",
        ResolutionKind::Tarball,
    );
    expect(
        &head,
        "sample",
        "1.0.0",
        "sha256:second",
        ResolutionKind::Tarball,
    );
    assert_eq!(diff(&base, &head).changed, ["sample"]);
}

#[test]
fn uv_rejects_invalid_git_url() {
    assert!(packages_for_diff(PackageManager::Uv, &fixture("uv", "invalid-git-url.lock")).is_err());
}

#[test]
fn bundler_submodules_and_pipenv_archive_subdirectories_affect_fingerprints() {
    for (manager, dir, stem) in [
        (PackageManager::Gemfile, "gemfile", "submodules"),
        (PackageManager::Pipfile, "pipfile", "archive-subdirectory"),
    ] {
        let base = packages_for_diff(manager, &fixture(dir, &format!("{stem}-base.lock"))).unwrap();
        let head = packages_for_diff(manager, &fixture(dir, &format!("{stem}-head.lock"))).unwrap();
        assert_eq!(diff(&base, &head).changed, ["sample"]);
    }
}

#[test]
fn uv_rejects_a_present_source_without_a_known_kind() {
    assert!(packages_for_diff(PackageManager::Uv, &fixture("uv", "unknown-source.lock")).is_err());
}
