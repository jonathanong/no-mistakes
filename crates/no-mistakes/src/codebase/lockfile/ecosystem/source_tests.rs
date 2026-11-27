use super::super::{diff, packages_for_diff, PackageManager, ResolutionKind, ResolvedPackage};
use std::path::PathBuf;

fn fixture(dir: &str, name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/lockfile")
        .join(dir)
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn expect(
    packages: &[ResolvedPackage],
    name: &str,
    version: &str,
    fingerprint: &str,
    kind: ResolutionKind,
) {
    let package = packages
        .iter()
        .find(|package| package.name == name)
        .unwrap_or_else(|| panic!("missing {name}"));
    assert_eq!(package.version, version, "{name}");
    assert_eq!(package.fingerprint, fingerprint, "{name}");
    assert_eq!(package.kind, kind, "{name}");
}

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
fn checked_gemfile_rejects_conflict_markers() {
    assert!(packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "conflicted.lock")
    )
    .is_err());
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
    expect(
        &packages,
        "example.invalid/raw",
        "v2.3.4",
        "",
        ResolutionKind::Registry,
    );
    assert!(packages_for_diff(PackageManager::Go, &fixture("gomod", "extra-token.mod")).is_err());
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
    assert_eq!(diff(&base, &head).changed, ["sample-local", "sample-sdist"]);
    assert!(packages_for_diff(PackageManager::Uv, &fixture("uv", "missing-version.lock")).is_err());
    assert!(packages_for_diff(PackageManager::Uv, &fixture("uv", "invalid-source.lock")).is_err());
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
fn checked_text_lockfiles_reject_unrecognized_top_level_structures() {
    assert!(packages_for_diff(
        PackageManager::Gemfile,
        &fixture("gemfile", "unrecognized-header.lock")
    )
    .is_err());
    for name in ["unknown-directive.mod", "conflict.mod"] {
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
    for name in ["empty-module-block.mod", "duplicate-module-block.mod"] {
        assert!(packages_for_diff(PackageManager::Go, &fixture("gomod", name)).is_err());
    }
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
