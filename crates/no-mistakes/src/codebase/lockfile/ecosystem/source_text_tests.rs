#[test]
fn checked_gemfile_rejects_conflict_markers() {
    assert!(packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "conflicted.lock")
    )
    .is_err());
}

#[test]
fn go_replace_keeps_a_double_slash_inside_a_quoted_path() {
    let packages =
        packages_for_diff(PackageManager::Go, &fixture("gomod", "quoted-slash.mod")).unwrap();
    expect(
        &packages,
        "example.invalid/lib",
        "v1.2.3",
        "",
        ResolutionKind::Registry,
    );
}

#[test]
fn go_requirements_allow_tabs_and_quoted_tokens_but_reject_extra_tokens() {
    let packages = packages_for_diff(PackageManager::Go, &fixture("gomod", "quoted.mod")).unwrap();
    expect(
        &packages,
        "example.invalid/quoted",
        "v1.2.3",
        "",
        ResolutionKind::Registry,
    );
    assert!(packages_for_diff(PackageManager::Go, &fixture("gomod", "extra-token.mod")).is_err());
    assert!(packages_for_diff(PackageManager::Go, &fixture("gomod", "backtick.mod")).is_err());
}

#[test]
fn checked_text_lockfiles_reject_unrecognized_top_level_structures() {
    assert!(packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "unrecognized-header.lock")
    )
    .is_err());
    for name in [
        "unknown-directive.mod",
        "conflict.mod",
        "invalid-replacement.mod",
        "excess-replacement-operands.mod",
        "excess-replacement-target.mod",
        "multiple-replacement-arrows.mod",
    ] {
        assert!(packages_for_diff(PackageManager::Go, &fixture("gomod", name)).is_err());
    }
    assert!(packages_for_diff(
        PackageManager::Pipfile,
        &fixture("pipfile", "invalid-editable.lock")
    )
    .is_err());
    let packages =
        packages_for_diff(PackageManager::Go, &fixture("gomod", "ignored-blocks.mod")).unwrap();
    assert_eq!(packages.len(), 1);
    expect(
        &packages,
        "example.invalid/library",
        "v1.2.3",
        "",
        ResolutionKind::Registry,
    );
}

#[test]
fn gemfile_reads_plugin_source_specs() {
    let base = packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "plugin-base.lock"),
    )
    .unwrap();
    expect(
        &base,
        "sample-plugin-gem",
        "1.0.0",
        "https://packages.example.invalid",
        ResolutionKind::Other,
    );
    let head = packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "plugin-head.lock"),
    )
    .unwrap();
    assert_eq!(diff(&base, &head).changed, ["sample-plugin-gem"]);
}

#[test]
fn go_accepts_one_valid_module_path_in_a_block() {
    let packages =
        packages_for_diff(PackageManager::Go, &fixture("gomod", "module-block.mod")).unwrap();
    expect(
        &packages,
        "example.invalid/library",
        "v1.0.0",
        "",
        ResolutionKind::Registry,
    );
    for name in [
        "empty-module-block.mod",
        "duplicate-module-block.mod",
        "repeated-module.mod",
        "repeated-module-block.mod",
    ] {
        assert!(packages_for_diff(PackageManager::Go, &fixture("gomod", name)).is_err());
    }
}

#[test]
fn source_selectors_change_packages_without_changing_the_pin_or_hash() {
    let cases = [
        (
            PackageManager::Pipfile,
            "pipfile",
            "subdirectory",
            "abc123#subdirectory=packages/sample",
            ResolutionKind::Git,
        ),
        (
            PackageManager::Pipfile,
            "pipfile",
            "hg-subdirectory",
            "https://example.invalid/sample#abc123#subdirectory=packages/sample",
            ResolutionKind::Other,
        ),
        (
            PackageManager::Uv,
            "uv",
            "archive-subdirectory",
            "sha256:same#subdirectory=packages/sample",
            ResolutionKind::Tarball,
        ),
        (
            PackageManager::Gemfile,
            "gemfile",
            "git-glob",
            "abc123#glob=packages/sample/sample.gemspec",
            ResolutionKind::Git,
        ),
        (
            PackageManager::Gemfile,
            "gemfile",
            "path-glob",
            "./sample#glob=packages/alpha/sample.gemspec",
            ResolutionKind::Directory,
        ),
    ];
    for (manager, dir, stem, fingerprint, kind) in cases {
        let base = packages_for_diff(manager, &fixture(dir, &format!("{stem}-base.lock"))).unwrap();
        let head = packages_for_diff(manager, &fixture(dir, &format!("{stem}-head.lock"))).unwrap();
        expect(
            &base,
            "sample",
            if manager == PackageManager::Pipfile {
                ""
            } else {
                "1.0.0"
            },
            fingerprint,
            kind,
        );
        assert_eq!(diff(&base, &head).changed, ["sample"]);
    }
}

#[test]
fn go_requires_a_module_and_retains_requested_revision_names() {
    assert!(
        packages_for_diff(PackageManager::Go, &fixture("gomod", "missing-module.mod")).is_err()
    );
    let packages = packages_for_diff(
        PackageManager::Go,
        &fixture("gomod", "requested-revision.mod"),
    )
    .unwrap();
    expect(
        &packages,
        "example.invalid/library",
        "release/sample",
        "",
        ResolutionKind::Registry,
    );
}

#[test]
fn go_accepts_inline_empty_require_blocks() {
    let packages = packages_for_diff(
        PackageManager::Go,
        &fixture("gomod", "inline-empty-block.mod"),
    )
    .unwrap();
    assert!(packages.is_empty());
}
