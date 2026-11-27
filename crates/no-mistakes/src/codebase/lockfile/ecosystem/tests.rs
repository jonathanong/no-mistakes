use super::super::validate::validate_for_planning;
use super::super::{
    detect_manager, diff, packages_for_diff, parse_lockfile, PackageManager, ResolutionKind,
    ResolvedPackage,
};
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

fn assert_diff(
    manager: PackageManager,
    dir: &str,
    base: &str,
    head: &str,
    added: &[&str],
    removed: &[&str],
    changed: &[&str],
) {
    let old = packages_for_diff(manager, &fixture(dir, base)).unwrap();
    let new = packages_for_diff(manager, &fixture(dir, head)).unwrap();
    let diff = diff(&old, &new);
    assert_eq!(diff.added, added, "{dir} added");
    assert_eq!(diff.removed, removed, "{dir} removed");
    assert_eq!(diff.changed, changed, "{dir} changed");
}

fn assert_round_trip(manager: PackageManager, dir: &str, name: &str) {
    let content = fixture(dir, name);
    let lenient = parse_lockfile(manager, &content);
    let checked = packages_for_diff(manager, &content).unwrap();
    assert_eq!(lenient, checked);
}

#[test]
fn cargo_packages_keep_source_kind_and_checksum() {
    let packages =
        packages_for_diff(PackageManager::Cargo, &fixture("cargo", "base.lock")).unwrap();
    assert_eq!(packages.len(), 7);
    expect(
        &packages,
        "sample-alpha",
        "1.0.0",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-beta",
        "0.2.0",
        "git+https://example.invalid/sample-beta.git?rev=abc123#abc123",
        ResolutionKind::Git,
    );
    expect(
        &packages,
        "sample-app",
        "0.1.0",
        "",
        ResolutionKind::Workspace,
    );
    expect(
        &packages,
        "sample-path",
        "0.3.0",
        "",
        ResolutionKind::Directory,
    );
    expect(
        &packages,
        "sample-sparse",
        "1.4.0",
        "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-other",
        "0.5.0",
        "other+https://example.invalid/sample-other",
        ResolutionKind::Other,
    );
    expect(
        &packages,
        "sample-digest",
        "1.0.0",
        "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        ResolutionKind::Registry,
    );
    assert_diff(
        PackageManager::Cargo,
        "cargo",
        "base.lock",
        "head.lock",
        &["sample-gamma"],
        &["sample-beta"],
        &["sample-alpha", "sample-digest"],
    );
}

#[test]
fn poetry_dependencies_table_keeps_the_package() {
    let packages =
        packages_for_diff(PackageManager::Poetry, &fixture("poetry", "base.lock")).unwrap();
    assert_eq!(packages.len(), 6);
    expect(
        &packages,
        "sample-alpha",
        "1.2.3",
        "sha256:aaaa",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-git",
        "0.9.0",
        "abc123",
        ResolutionKind::Git,
    );
    expect(
        &packages,
        "sample-dir",
        "0.1.0",
        "lib/sample-dir",
        ResolutionKind::Directory,
    );
    expect(
        &packages,
        "sample-url",
        "0.2.0",
        "",
        ResolutionKind::Tarball,
    );
    expect(
        &packages,
        "sample-legacy",
        "0.3.0",
        "",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-meta",
        "1.0.0",
        "sha256:eeee",
        ResolutionKind::Registry,
    );
    assert_diff(
        PackageManager::Poetry,
        "poetry",
        "base.lock",
        "head.lock",
        &["sample-gamma"],
        &["sample-git"],
        &["sample-alpha"],
    );
}

#[test]
fn uv_inline_sources_set_kind_and_revision() {
    let packages = packages_for_diff(PackageManager::Uv, &fixture("uv", "base.lock")).unwrap();
    assert_eq!(packages.len(), 3);
    expect(
        &packages,
        "sample-app",
        "0.1.0",
        ".",
        ResolutionKind::Directory,
    );
    expect(
        &packages,
        "sample-alpha",
        "1.2.3",
        "sha256:aaaa",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-git",
        "0.4.0",
        "abc123",
        ResolutionKind::Git,
    );
    assert_diff(
        PackageManager::Uv,
        "uv",
        "base.lock",
        "head.lock",
        &["sample-gamma"],
        &["sample-git"],
        &["sample-alpha"],
    );
}

#[test]
fn pipfile_reads_default_and_develop() {
    let packages =
        packages_for_diff(PackageManager::Pipfile, &fixture("pipfile", "base.lock")).unwrap();
    assert_eq!(packages.len(), 3);
    expect(
        &packages,
        "sample-alpha",
        "1.2.3",
        "sha256:aaaa",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-git",
        "0.9.0",
        "abc123",
        ResolutionKind::Git,
    );
    expect(
        &packages,
        "sample-path",
        "0.1.0",
        "./sample-path",
        ResolutionKind::Directory,
    );
    assert!(packages.iter().all(|package| package.name != "_meta"));
    assert_diff(
        PackageManager::Pipfile,
        "pipfile",
        "base.lock",
        "head.lock",
        &["sample-gamma"],
        &["sample-git"],
        &["sample-alpha"],
    );
}

#[test]
fn composer_dist_source_and_path() {
    let packages =
        packages_for_diff(PackageManager::Composer, &fixture("composer", "base.lock")).unwrap();
    assert_eq!(packages.len(), 3);
    expect(
        &packages,
        "acme/sample-alpha",
        "1.2.3",
        "aaaa",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "acme/sample-git",
        "0.9.0",
        "abc123",
        ResolutionKind::Git,
    );
    expect(
        &packages,
        "acme/sample-path",
        "0.1.0",
        "../sample-path",
        ResolutionKind::Directory,
    );
    assert_diff(
        PackageManager::Composer,
        "composer",
        "base.lock",
        "head.lock",
        &["acme/sample-gamma"],
        &["acme/sample-git"],
        &["acme/sample-alpha"],
    );
}

#[test]
fn gomod_reads_require_lines_not_replace_or_exclude() {
    let packages = packages_for_diff(PackageManager::Go, &fixture("gomod", "base.mod")).unwrap();
    let names: Vec<_> = packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec![
            "example.com/sample-alpha",
            "example.com/sample-beta",
            "example.com/sample-gamma",
        ]
    );
    expect(
        &packages,
        "example.com/sample-alpha",
        "v1.2.3",
        "",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "example.com/sample-beta",
        "v0.4.0",
        "",
        ResolutionKind::Registry,
    );
    assert!(packages
        .iter()
        .all(|package| package.name != "../sample-beta"));
    assert_diff(
        PackageManager::Go,
        "gomod",
        "base.mod",
        "head.mod",
        &["example.com/sample-delta"],
        &["example.com/sample-beta"],
        &["example.com/sample-alpha"],
    );
    assert_eq!(detect_manager("go.sum"), None);
}

#[test]
fn gemfile_specs_checksums_and_git_revision() {
    let packages =
        packages_for_diff(PackageManager::Gemfile, &fixture("gemfile", "base.lock")).unwrap();
    assert_eq!(packages.len(), 4);
    expect(
        &packages,
        "sample-alpha",
        "1.2.3",
        "aaaa",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-beta",
        "0.4.0",
        "bbbb",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-app",
        "0.1.0",
        "vendor/sample-app",
        ResolutionKind::Directory,
    );
    expect(
        &packages,
        "sample-git",
        "0.9.0",
        "abc123",
        ResolutionKind::Git,
    );
    assert!(packages.iter().all(|package| package.name != "sample-dep"));
    assert_diff(
        PackageManager::Gemfile,
        "gemfile",
        "base.lock",
        "head.lock",
        &["sample-gamma"],
        &["sample-beta"],
        &["sample-alpha", "sample-app"],
    );
}

#[test]
fn malformed_ecosystem_lockfiles_are_not_empty_successes() {
    let cases = [
        (
            PackageManager::Cargo,
            "cargo",
            "malformed.lock",
            "empty.lock",
            "lockfile-cargo-malformed",
        ),
        (
            PackageManager::Poetry,
            "poetry",
            "malformed.lock",
            "empty.lock",
            "lockfile-poetry-malformed",
        ),
        (
            PackageManager::Uv,
            "uv",
            "malformed.lock",
            "empty.lock",
            "lockfile-uv-malformed",
        ),
        (
            PackageManager::Pipfile,
            "pipfile",
            "malformed.lock",
            "empty.lock",
            "lockfile-pipfile-malformed",
        ),
        (
            PackageManager::Go,
            "gomod",
            "malformed.mod",
            "empty.mod",
            "lockfile-gomod-malformed",
        ),
        (
            PackageManager::Gemfile,
            "gemfile",
            "malformed.lock",
            "empty.lock",
            "lockfile-gemfile-malformed",
        ),
        (
            PackageManager::Composer,
            "composer",
            "malformed.lock",
            "empty.lock",
            "lockfile-composer-malformed",
        ),
    ];
    for (manager, dir, malformed, empty, kind) in cases {
        let bad = fixture(dir, malformed);
        assert!(parse_lockfile(manager, &bad).is_empty(), "{dir} lenient");
        assert!(packages_for_diff(manager, &bad).is_err(), "{dir} checked");
        assert_eq!(validate_for_planning(manager, &bad).unwrap_err().kind, kind);
        let empty = fixture(dir, empty);
        assert!(parse_lockfile(manager, &empty).is_empty(), "{dir} empty");
        assert!(
            validate_for_planning(manager, &empty).is_ok(),
            "{dir} empty"
        );
        assert_round_trip(manager, dir, empty_name(dir));
        assert!(
            validate_for_planning(manager, "not a lockfile").is_err(),
            "{dir} garbage"
        );
    }
}

fn empty_name(dir: &str) -> &'static str {
    if dir == "gomod" {
        "empty.mod"
    } else {
        "empty.lock"
    }
}

#[test]
fn pipfile_and_composer_reject_entries_without_identity() {
    let pipfile = r#"{"default":{"sample-alpha":{"hashes":["sha256:aaaa"]}},"develop":{}}"#;
    assert_eq!(
        validate_for_planning(PackageManager::Pipfile, pipfile)
            .unwrap_err()
            .kind,
        "lockfile-pipfile-malformed"
    );
    let composer = r#"{"packages":[{"version":"1.0.0"}],"packages-dev":[]}"#;
    assert_eq!(
        validate_for_planning(PackageManager::Composer, composer)
            .unwrap_err()
            .kind,
        "lockfile-composer-malformed"
    );
    let gomod = "module example.com/sample\n\nrequire example.com/sample-alpha 1.2.3\n";
    assert!(validate_for_planning(PackageManager::Go, gomod).is_err());
}

#[test]
fn quoted_hash_without_equals_is_not_a_fingerprint() {
    let content = "[[package]]\nname = \"sample-alpha\"\nversion = \"1.0.0\"\ndescription = \"synthetic hash\"\n";
    let packages = packages_for_diff(PackageManager::Poetry, content).unwrap();
    expect(
        &packages,
        "sample-alpha",
        "1.0.0",
        "",
        ResolutionKind::Registry,
    );
}
