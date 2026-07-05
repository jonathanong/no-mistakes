use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/lockfile/pnpm")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e))
}

#[test]
fn parse_basic_packages() {
    let pkgs = parse(&fixture("sample.yaml"));
    let find = |name: &str| pkgs.iter().find(|p| p.name == name);

    let lodash = find("lodash").unwrap();
    assert_eq!(lodash.version, "4.17.21");
    assert_eq!(lodash.fingerprint, "sha512-abc123");
    assert_eq!(lodash.kind, ResolutionKind::Registry);

    let scoped = find("@scope/pkg").unwrap();
    assert_eq!(scoped.version, "1.0.0");
    assert_eq!(scoped.kind, ResolutionKind::Registry);
}

#[test]
fn parse_git_resolution() {
    let pkgs = parse(&fixture("sample.yaml"));
    let git = pkgs
        .iter()
        .find(|p| p.name == "github.com/org/repo")
        .unwrap();
    assert_eq!(git.fingerprint, "abc123");
    assert_eq!(git.kind, ResolutionKind::Git);
}

#[test]
fn parse_tarball_resolution() {
    let pkgs = parse(&fixture("sample.yaml"));
    let tb = pkgs.iter().find(|p| p.name == "some-tarball").unwrap();
    assert_eq!(tb.fingerprint, "https://example.com/pkg.tgz");
    assert_eq!(tb.kind, ResolutionKind::Tarball);
}

#[test]
fn parse_directory_resolution() {
    let pkgs = parse(&fixture("sample.yaml"));
    let dir = pkgs.iter().find(|p| p.name == "local-pkg").unwrap();
    assert_eq!(dir.fingerprint, "../local-pkg");
    assert_eq!(dir.kind, ResolutionKind::Directory);
}

#[test]
fn parse_commit_only() {
    let pkgs = parse(&fixture("sample.yaml"));
    let c = pkgs.iter().find(|p| p.name == "commit-only").unwrap();
    assert_eq!(c.fingerprint, "deadbeef");
    assert_eq!(c.kind, ResolutionKind::Git);
}

#[test]
fn parse_no_resolution() {
    let pkgs = parse(&fixture("sample.yaml"));
    let nr = pkgs.iter().find(|p| p.name == "no-resolution").unwrap();
    assert_eq!(nr.fingerprint, "");
    assert_eq!(nr.kind, ResolutionKind::Other);
}

#[test]
fn parse_empty_content() {
    assert!(parse("").is_empty());
}

#[test]
fn parse_no_packages_section() {
    assert!(parse("lockfileVersion: '9.0'\n").is_empty());
}

#[test]
fn parse_packages_with_importers_fixture() {
    let pkgs = parse(&fixture("importers.yaml"));
    let lodash = pkgs.iter().find(|p| p.name == "lodash").unwrap();
    assert_eq!(lodash.version, "4.17.21");
    assert_eq!(lodash.fingerprint, "sha512-abc123");
    assert_eq!(lodash.kind, ResolutionKind::Registry);
}

#[test]
fn parse_importers_groups_by_dependency_type() {
    let importers = parse_importers(&fixture("importers.yaml"));
    assert_eq!(importers.len(), 2);
    assert_eq!(importers[0].path, ".");
    assert_eq!(importers[1].path, "packages/app");

    let app = importers.iter().find(|i| i.path == "packages/app").unwrap();
    assert_eq!(app.dependencies.len(), 1);
    assert_eq!(app.dependencies[0].alias, "lodash");
    assert_eq!(app.dependencies[0].specifier, "^4.17.21");
    assert_eq!(app.dependencies[0].version, "4.17.21");
    assert_eq!(app.dependencies[0].resolution_name, None);

    assert_eq!(app.dev_dependencies.len(), 2);
    assert_eq!(app.dev_dependencies[0].alias, "chalk");
    assert_eq!(app.dev_dependencies[0].resolution_name, None);
    assert_eq!(app.dev_dependencies[1].alias, "image-lib");
    assert_eq!(
        app.dev_dependencies[1].resolution_name.as_deref(),
        Some("sharp")
    );
    assert_eq!(app.optional_dependencies.len(), 1);
    assert_eq!(app.optional_dependencies[0].alias, "@scope/optional");
}

#[test]
fn parse_importers_empty_content() {
    assert!(parse_importers("").is_empty());
}

#[test]
fn parse_importers_missing_importers_section() {
    assert!(parse_importers("packages:\n  lodash@4.17.21: {}\n").is_empty());
}

#[test]
fn parse_importers_invalid_yaml() {
    assert!(parse_importers("{ invalid: yaml: [[[").is_empty());
}

#[test]
fn parse_importers_skips_empty_importer_and_dependency_keys() {
    let importers = parse_importers(
        "importers:\n  ~:\n    dependencies:\n      lodash: 4.17.21\n  packages/app:\n    dependencies:\n      ~: 1.0.0\n      lodash: 4.17.21\n",
    );

    assert_eq!(importers.len(), 1);
    assert_eq!(importers[0].path, "packages/app");
    assert_eq!(importers[0].dependencies.len(), 1);
    assert_eq!(importers[0].dependencies[0].alias, "lodash");
}

#[test]
fn parse_importers_accepts_string_and_other_dependency_shapes() {
    let importers = parse_importers(
        "importers:\n  packages/app:\n    dependencies:\n      string-form: '1.0.0'\n      object-form:\n        specifier: 'npm:'\n        version: '2.0.0'\n      number-form: 3\n",
    );
    let deps = &importers[0].dependencies;

    assert_eq!(deps.len(), 3);
    assert_eq!(deps[0].alias, "number-form");
    assert_eq!(deps[0].specifier, "");
    assert_eq!(deps[0].version, "");
    assert_eq!(deps[1].alias, "object-form");
    assert_eq!(deps[1].resolution_name, None);
    assert_eq!(deps[2].alias, "string-form");
    assert_eq!(deps[2].specifier, "");
    assert_eq!(deps[2].version, "1.0.0");
}

#[test]
fn parse_invalid_yaml() {
    assert!(parse("{ invalid: yaml: [[[").is_empty());
}

#[test]
fn split_scoped_package() {
    let (name, ver) = split_name_version("@scope/pkg@1.2.3");
    assert_eq!(name, "@scope/pkg");
    assert_eq!(ver, "1.2.3");
}

#[test]
fn split_regular_package() {
    let (name, ver) = split_name_version("lodash@4.17.21");
    assert_eq!(name, "lodash");
    assert_eq!(ver, "4.17.21");
}

#[test]
fn split_no_version() {
    let (name, ver) = split_name_version("github.com/org/repo");
    assert_eq!(name, "github.com/org/repo");
    assert_eq!(ver, "");
}

#[test]
fn yaml_key_number() {
    let val = serde_yaml::Value::Number(serde_yaml::Number::from(42));
    assert_eq!(yaml_key_to_string(&val), "42");
}

#[test]
fn yaml_key_bool() {
    let val = serde_yaml::Value::Bool(true);
    assert_eq!(yaml_key_to_string(&val), "true");
}

#[test]
fn yaml_key_null() {
    let val = serde_yaml::Value::Null;
    assert_eq!(yaml_key_to_string(&val), "");
}

#[test]
fn resolution_info_no_resolution_field() {
    let val = serde_yaml::Value::Null;
    let (fp, kind) = resolution_info(&val);
    assert_eq!(fp, "");
    assert_eq!(kind, ResolutionKind::Other);
}

#[test]
fn resolution_info_unknown_keys() {
    let content = "packages:\n  exotic@1.0.0:\n    resolution: {checksum: abc123}\n";
    let pkgs = parse(content);
    assert_eq!(pkgs[0].kind, ResolutionKind::Other);
    assert_eq!(pkgs[0].fingerprint, "");
}

#[test]
fn parse_tarball_with_integrity_prefers_integrity() {
    // When both tarball and integrity are present, integrity is the fingerprint
    // so that a content change at the same URL is detected.
    let content = "packages:\n  some-pkg@1.0.0:\n    resolution: {tarball: 'https://example.com/pkg.tgz', integrity: sha512-newhash}\n";
    let pkgs = parse(content);
    assert_eq!(pkgs[0].fingerprint, "sha512-newhash");
    assert_eq!(pkgs[0].kind, ResolutionKind::Tarball);
}

#[test]
fn split_v5_leading_slash() {
    // pnpm v5/v6 lockfiles use /lodash@4.17.21 with a leading slash
    let (name, ver) = split_name_version("/lodash@4.17.21");
    assert_eq!(name, "lodash");
    assert_eq!(ver, "4.17.21");
}

#[test]
fn split_v5_scoped_leading_slash() {
    // pnpm v5/v6 scoped package: /@scope/pkg@1.0.0
    let (name, ver) = split_name_version("/@scope/pkg@1.0.0");
    assert_eq!(name, "@scope/pkg");
    assert_eq!(ver, "1.0.0");
}

#[test]
fn split_v5_slash_separated_unscoped() {
    // pnpm v5 slash-only format: /lodash/4.17.21 (no @ separator)
    let (name, ver) = split_name_version("/lodash/4.17.21");
    assert_eq!(name, "lodash");
    assert_eq!(ver, "4.17.21");
}

#[test]
fn split_v5_slash_separated_scoped() {
    // pnpm v5 scoped slash format: /@scope/pkg/1.0.0
    let (name, ver) = split_name_version("/@scope/pkg/1.0.0");
    assert_eq!(name, "@scope/pkg");
    assert_eq!(ver, "1.0.0");
}

#[test]
fn split_v5_slash_separated_strips_peer_suffix() {
    // pnpm v5 peer dep suffix uses `_`: /lodash/4.17.21_react@18
    let (name, ver) = split_name_version("/lodash/4.17.21_react@18");
    assert_eq!(name, "lodash");
    assert_eq!(ver, "4.17.21");
}

#[test]
fn split_scoped_no_slash() {
    // Bare scope key with no package name: @scope (unlikely but defensive)
    let (name, ver) = split_name_version("@scope");
    assert_eq!(name, "@scope");
    assert_eq!(ver, "");
}

#[test]
fn split_v5_scoped_with_peer_suffix() {
    // pnpm v5 scoped format with peer dep suffix: /@scope/pkg/1.0.0_react@18
    let (name, ver) = split_name_version("/@scope/pkg/1.0.0_react@18");
    assert_eq!(name, "@scope/pkg");
    assert_eq!(ver, "1.0.0");
}

#[test]
fn split_scoped_no_version() {
    // Scoped key with no version and no extra slash: @scope/pkg
    let (name, ver) = split_name_version("@scope/pkg");
    assert_eq!(name, "@scope/pkg");
    assert_eq!(ver, "");
}
