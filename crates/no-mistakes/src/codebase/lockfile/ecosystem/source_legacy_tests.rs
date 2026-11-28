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

#[test]
fn composer_compares_unpinned_distribution_urls() {
    let base = packages_for_diff(
        PackageManager::Composer,
        &fixture("composer", "archive-url-base.lock"),
    )
    .unwrap();
    let head = packages_for_diff(
        PackageManager::Composer,
        &fixture("composer", "archive-url-head.lock"),
    )
    .unwrap();
    assert_eq!(diff(&base, &head).changed, ["example/sample"]);
}

#[test]
fn cargo_and_uv_reject_nonpositive_lockfile_format_markers() {
    for (manager, dir) in [(PackageManager::Cargo, "cargo"), (PackageManager::Uv, "uv")] {
        for name in ["negative-version.lock", "zero-version.lock"] {
            assert!(packages_for_diff(manager, &fixture(dir, name)).is_err());
        }
    }
}

#[test]
fn gemfile_rejects_a_misindented_spec_and_reads_algorithm_checksums() {
    assert!(packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "bad-indent.lock")
    )
    .is_err());
    let base = packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "checksum-algo-base.lock"),
    )
    .unwrap();
    expect(
        &base,
        "sample-sha",
        "1.0.0",
        "sha512=abcd",
        ResolutionKind::Registry,
    );
    expect(
        &base,
        "sample-list",
        "1.0.0",
        "sha256=2222,sha512=1111",
        ResolutionKind::Registry,
    );
    assert!(base.iter().all(|package| package.name != "sample-dep"));
    let reordered = packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "checksum-algo-reordered.lock"),
    )
    .unwrap();
    assert_eq!(base, reordered);
    let head = packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "checksum-algo-head.lock"),
    )
    .unwrap();
    assert_eq!(diff(&base, &head).changed, ["sample-sha"]);
}

#[test]
fn uv_requires_a_format_marker_and_a_git_pin() {
    for name in [
        "missing-format-version.lock",
        "unpinned-git.lock",
        "empty-rev.lock",
    ] {
        assert!(packages_for_diff(PackageManager::Uv, &fixture("uv", name)).is_err());
    }
    assert!(packages_for_diff(PackageManager::Uv, &fixture("uv", "empty.lock")).is_ok());
    let packages =
        packages_for_diff(PackageManager::Uv, &fixture("uv", "empty-fragment.lock")).unwrap();
    expect(&packages, "sample", "1.0.0", "abc123", ResolutionKind::Git);
}

#[test]
fn lockfile_parsers_reject_empty_identity_fields() {
    for (manager, dir, name) in [
        (PackageManager::Composer, "composer", "empty-dist.lock"),
        (
            PackageManager::Composer,
            "composer",
            "dist-missing-url.lock",
        ),
        (PackageManager::Gemfile, "gemfile", "bad-version.lock"),
        (PackageManager::Uv, "uv", "bad-name.lock"),
        (PackageManager::Uv, "uv", "empty-package-version.lock"),
    ] {
        assert!(
            packages_for_diff(manager, &fixture(dir, name)).is_err(),
            "{dir}/{name}"
        );
    }
    let packages = packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "unknown-header.lock"),
    )
    .unwrap();
    expect(
        &packages,
        "sample-alpha",
        "1.2.3",
        "",
        ResolutionKind::Registry,
    );
    assert!(packages.iter().all(|package| package.name != "phantom"));
    let versions = packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "prerelease.lock"),
    )
    .unwrap();
    expect(
        &versions,
        "sample-pre",
        "1.0.0.pre.1",
        "",
        ResolutionKind::Registry,
    );
    expect(
        &versions,
        "sample-platform",
        "1.2.3-x86_64-linux",
        "",
        ResolutionKind::Registry,
    );
    let directory = packages_for_diff(
        PackageManager::Uv,
        &fixture("uv", "versionless-directory.lock"),
    )
    .unwrap();
    expect(
        &directory,
        "sample-app",
        "",
        "lib/sample",
        ResolutionKind::Directory,
    );
}

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
