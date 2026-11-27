#[test]
fn poetry_commit_change_uses_resolved_reference() {
    let base =
        packages_for_diff(PackageManager::Poetry, &fixture("poetry", "pin-base.lock")).unwrap();
    expect(&base, "sample-pin", "1.0.0", "aaa111", ResolutionKind::Git);
    expect(&base, "sample-ref", "0.1.0", "ccc333", ResolutionKind::Git);
    let head =
        packages_for_diff(PackageManager::Poetry, &fixture("poetry", "pin-head.lock")).unwrap();
    let diff = diff(&base, &head);
    assert_eq!(diff.added, Vec::<String>::new());
    assert_eq!(diff.removed, Vec::<String>::new());
    assert_eq!(diff.changed, vec!["sample-pin".to_string()]);
}

#[test]
fn uv_fragment_outranks_rev_and_virtual_omits_version() {
    let base = packages_for_diff(PackageManager::Uv, &fixture("uv", "pin-base.lock")).unwrap();
    assert_eq!(base.len(), 5);
    expect(&base, "sample-app", "", ".", ResolutionKind::Directory);
    expect(
        &base,
        "sample-dir",
        "0.1.0",
        "lib/sample-dir",
        ResolutionKind::Directory,
    );
    expect(
        &base,
        "sample-edit",
        "",
        "lib/sample-edit",
        ResolutionKind::Directory,
    );
    expect(&base, "sample-git", "0.4.0", "aaa111", ResolutionKind::Git);
    expect(&base, "sample-rev", "0.2.0", "ddd444", ResolutionKind::Git);
    let head = packages_for_diff(PackageManager::Uv, &fixture("uv", "pin-head.lock")).unwrap();
    let diff = diff(&base, &head);
    assert_eq!(diff.changed, ["sample-dir", "sample-git"]);
}

#[test]
fn multiline_toml_string_does_not_invent_a_package() {
    let packages =
        packages_for_diff(PackageManager::Poetry, &fixture("poetry", "multiline.lock")).unwrap();
    assert_eq!(packages.len(), 1);
    expect(
        &packages,
        "sample-alpha",
        "1.0.0",
        "",
        ResolutionKind::Registry,
    );
    assert!(packages_for_diff(PackageManager::Cargo, &fixture("cargo", "syntax.lock")).is_err());
}

#[test]
fn poetry_directory_path_changes_and_local_archive_kinds_are_reported() {
    let base =
        packages_for_diff(PackageManager::Poetry, &fixture("poetry", "path-base.lock")).unwrap();
    let head =
        packages_for_diff(PackageManager::Poetry, &fixture("poetry", "path-head.lock")).unwrap();
    expect(
        &base,
        "sample-archive",
        "1.0.0",
        "sha256:sample",
        ResolutionKind::Tarball,
    );
    assert_eq!(diff(&base, &head).changed, ["sample-directory"]);
}

#[test]
fn uv_uses_source_hash_without_wheels_and_classifies_direct_archives() {
    let base = packages_for_diff(
        PackageManager::Uv,
        &fixture("uv", "distributions-base.lock"),
    )
    .unwrap();
    let head = packages_for_diff(
        PackageManager::Uv,
        &fixture("uv", "distributions-head.lock"),
    )
    .unwrap();
    expect(
        &base,
        "sample-archive",
        "1.0.0",
        "https://packages.example.invalid/example.tar.gz",
        ResolutionKind::Tarball,
    );
    expect(
        &head,
        "sample-local",
        "1.0.0",
        "sha256:same",
        ResolutionKind::Tarball,
    );
    assert_eq!(
        diff(&base, &head).changed,
        ["sample-local", "sample-sdist", "sample-wheel"]
    );
    assert!(packages_for_diff(PackageManager::Uv, &fixture("uv", "missing-version.lock")).is_err());
    assert!(packages_for_diff(PackageManager::Uv, &fixture("uv", "invalid-source.lock")).is_err());
}

#[test]
fn python_git_subdirectories_contribute_to_dependency_identity() {
    for (manager, dir) in [
        (PackageManager::Uv, "uv"),
        (PackageManager::Poetry, "poetry"),
    ] {
        let base = packages_for_diff(manager, &fixture(dir, "subdirectory-base.lock")).unwrap();
        let head = packages_for_diff(manager, &fixture(dir, "subdirectory-head.lock")).unwrap();
        expect(
            &base,
            "sample",
            "1.0.0",
            "abc123#subdirectory=packages/sample",
            ResolutionKind::Git,
        );
        assert_eq!(diff(&base, &head).changed, ["sample"]);
    }
    assert!(
        packages_for_diff(PackageManager::Uv, &fixture("uv", "invalid-registry.lock")).is_err()
    );
}

#[test]
fn legacy_cargo_checksums_match_inline_checksums() {
    let old =
        packages_for_diff(PackageManager::Cargo, &fixture("cargo", "checksum-v1.lock")).unwrap();
    let new =
        packages_for_diff(PackageManager::Cargo, &fixture("cargo", "checksum-v2.lock")).unwrap();
    assert_eq!(old, new);
    assert!(diff(&old, &new).changed.is_empty());
}

#[test]
fn distribution_hash_changes_are_visible_and_reordering_is_ignored() {
    for (manager, dir, stem) in [
        (PackageManager::Pipfile, "pipfile", "artifact"),
        (PackageManager::Poetry, "poetry", "artifact"),
        (PackageManager::Poetry, "poetry", "legacy-artifact"),
        (PackageManager::Uv, "uv", "artifact"),
    ] {
        let packages = |variant| {
            packages_for_diff(manager, &fixture(dir, &format!("{stem}-{variant}.lock"))).unwrap()
        };
        let base = packages("base");
        assert_eq!(base, packages("reordered"), "{dir}/{stem}");
        assert_eq!(diff(&base, &packages("changed")).changed, ["sample"]);
    }
}
