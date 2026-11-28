use super::super::{packages_for_diff, PackageManager, ResolutionKind, ResolvedPackage};

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
          "dist": { "type": "path", "url": "../path-dist", "reference": "dist-ref" },
          "source": { "type": "path", "url": "../path-src", "reference": "src-ref" }
        },
        {
          "name": "acme/reg",
          "version": "2.0.0",
          "dist": { "type": "zip", "url": "https://example.invalid/reg.zip", "reference": "" },
          "source": { "type": "git", "url": "https://example.invalid/reg.git", "reference": "src-pin" }
        }
      ]
    }"#;
    let packages = packages_for_diff(PackageManager::Composer, content).unwrap();
    expect(
        &packages,
        "acme/path",
        "1.0.0",
        "../path-dist#dist-ref",
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
