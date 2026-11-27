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

include!("source_json_tests.rs");
include!("source_toml_tests.rs");
include!("source_text_tests.rs");
