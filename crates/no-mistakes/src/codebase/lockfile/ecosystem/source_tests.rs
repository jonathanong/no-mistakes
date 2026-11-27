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
    expect(&base, "sample-app", "", "", ResolutionKind::Directory);
    expect(&base, "sample-dir", "0.1.0", "", ResolutionKind::Directory);
    expect(&base, "sample-edit", "", "", ResolutionKind::Directory);
    expect(&base, "sample-git", "0.4.0", "aaa111", ResolutionKind::Git);
    expect(&base, "sample-rev", "0.2.0", "ddd444", ResolutionKind::Git);
    let head = packages_for_diff(PackageManager::Uv, &fixture("uv", "pin-head.lock")).unwrap();
    let diff = diff(&base, &head);
    assert_eq!(diff.changed, vec!["sample-git".to_string()]);
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
fn toml_without_a_lock_header_is_malformed() {
    for manager in [
        PackageManager::Cargo,
        PackageManager::Poetry,
        PackageManager::Uv,
    ] {
        assert!(
            packages_for_diff(manager, "# comment only\n").is_err(),
            "{manager:?}"
        );
    }
}

#[test]
fn uv_package_without_source_uses_the_wheel_hash() {
    let content = r#"
version = 1

[[package]]
name = "sample-reg"
version = "1.2.3"
wheels = [{ hash = "sha256:wheel" }]
"#;
    let packages = packages_for_diff(PackageManager::Uv, content).unwrap();
    expect(
        &packages,
        "sample-reg",
        "1.2.3",
        "sha256:wheel",
        ResolutionKind::Registry,
    );
}

#[test]
fn poetry_git_without_pins_uses_file_and_metadata_hashes() {
    let content = r#"
[[package]]
name = "sample-git"
version = "1.0.0"
source = { type = "git" }
files = [{ file = "a.tar.gz", hash = "sha256:from-file" }]

[[package]]
name = "sample-meta"
version = "2.0.0"

[[package]]
name = "sample-kept"
version = "3.0.0"
files = [{ file = "kept.tar.gz", hash = "sha256:kept" }]

[metadata.files]
sample-meta = [{ file = "meta.tar.gz" }]
sample-kept = [{ file = "kept.tar.gz", hash = "sha256:ignored" }]
"#;
    let packages = packages_for_diff(PackageManager::Poetry, content).unwrap();
    expect(
        &packages,
        "sample-git",
        "1.0.0",
        "sha256:from-file",
        ResolutionKind::Git,
    );
    expect(
        &packages,
        "sample-meta",
        "2.0.0",
        "",
        ResolutionKind::Registry,
    );
    expect(
        &packages,
        "sample-kept",
        "3.0.0",
        "sha256:kept",
        ResolutionKind::Registry,
    );
}

#[test]
fn pipfile_editable_without_path_and_a_missing_section() {
    let packages = packages_for_diff(
        PackageManager::Pipfile,
        r#"{"default":{"sample-edit":{"editable":true}}}"#,
    )
    .unwrap();
    expect(&packages, "sample-edit", "", "", ResolutionKind::Directory);
    assert!(packages_for_diff(PackageManager::Pipfile, r#"{"_meta":{}}"#).is_err());
}

#[test]
fn composer_path_reference_and_registry_source_reference() {
    let content = r#"{
      "packages": [
        {
          "name": "acme/path",
          "version": "1.0.0",
          "dist": { "type": "path", "reference": "dist-ref" },
          "source": { "type": "path", "reference": "src-ref" }
        },
        {
          "name": "acme/reg",
          "version": "2.0.0",
          "dist": { "type": "zip", "reference": "" },
          "source": { "type": "git", "url": "https://example.invalid/reg.git", "reference": "src-pin" }
        }
      ]
    }"#;
    let packages = packages_for_diff(PackageManager::Composer, content).unwrap();
    expect(
        &packages,
        "acme/path",
        "1.0.0",
        "dist-ref",
        ResolutionKind::Directory,
    );
    expect(
        &packages,
        "acme/reg",
        "2.0.0",
        "src-pin",
        ResolutionKind::Registry,
    );
    assert!(packages_for_diff(PackageManager::Composer, r#"{"content-hash":"x"}"#).is_err());
}

#[test]
fn gemfile_checksum_miss_and_spaced_name() {
    let content = "\
GEM
  specs:
    sample-alpha (1.2.3)

CHECKSUMS
  sample-alpha (1.2.3)
  sample-missing (9.9.9) sha256=zzzz
";
    let packages = packages_for_diff(PackageManager::Gemfile, content).unwrap();
    expect(
        &packages,
        "sample-alpha",
        "1.2.3",
        "",
        ResolutionKind::Registry,
    );
    assert!(packages_for_diff(
        PackageManager::Gemfile,
        "GEM\n  specs:\n    bad name (1.0.0)\n",
    )
    .is_err());
}
