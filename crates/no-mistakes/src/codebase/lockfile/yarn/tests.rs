use super::*;

const CLASSIC: &str = r#"# yarn lockfile v1

lodash@^4.17.0, lodash@^4.17.21:
  version "4.17.21"
  resolved "https://registry.yarnpkg.com/lodash/-/lodash-4.17.21.tgz"
  integrity sha512-abc123

"@scope/pkg@^1.0.0":
  version "1.0.0"
  resolved "https://registry.yarnpkg.com/@scope/pkg/-/@scope/pkg-1.0.0.tgz"
  integrity sha512-scoped
"#;

const BERRY: &str = r#"__metadata:
  version: 6

lodash@npm:^4.17.0:
  version: 4.17.21
  resolution: "lodash@npm:4.17.21"
  checksum: abc123
  languageName: node
  linkType: hard

"@scope/pkg@npm:^1.0.0":
  version: 1.0.0
  resolution: "@scope/pkg@npm:1.0.0"
  checksum: scoped456
  languageName: node
  linkType: hard
"#;

#[test]
fn parse_classic_basic() {
    let pkgs = parse(CLASSIC);
    let lodash = pkgs.iter().find(|p| p.name == "lodash").unwrap();
    assert_eq!(lodash.version, "4.17.21");
    assert_eq!(lodash.fingerprint, "sha512-abc123");
    assert_eq!(lodash.kind, ResolutionKind::Registry);
}

#[test]
fn parse_classic_scoped() {
    let pkgs = parse(CLASSIC);
    let scoped = pkgs.iter().find(|p| p.name == "@scope/pkg").unwrap();
    assert_eq!(scoped.version, "1.0.0");
}

#[test]
fn parse_classic_multiple_specifiers() {
    let pkgs = parse(CLASSIC);
    let lodash_entries: Vec<_> = pkgs.iter().filter(|p| p.name == "lodash").collect();
    assert_eq!(
        lodash_entries.len(),
        1,
        "multiple specifiers for same pkg = one entry"
    );
}

#[test]
fn parse_berry_basic() {
    let pkgs = parse(BERRY);
    let lodash = pkgs.iter().find(|p| p.name == "lodash").unwrap();
    assert_eq!(lodash.version, "4.17.21");
    assert_eq!(lodash.fingerprint, "abc123");
}

#[test]
fn parse_berry_skips_metadata() {
    let pkgs = parse(BERRY);
    assert!(pkgs.iter().all(|p| p.name != "__metadata"));
}

#[test]
fn parse_berry_scoped() {
    let pkgs = parse(BERRY);
    let scoped = pkgs.iter().find(|p| p.name == "@scope/pkg").unwrap();
    assert_eq!(scoped.version, "1.0.0");
    assert_eq!(scoped.fingerprint, "scoped456");
}

#[test]
fn parse_classic_empty() {
    assert!(parse("# yarn lockfile v1\n").is_empty());
}

#[test]
fn parse_classic_no_integrity_uses_resolved() {
    let content = "# yarn lockfile v1\n\npkg@^1.0.0:\n  version \"1.0.0\"\n  resolved \"https://example.com/pkg.tgz\"\n";
    let pkgs = parse(content);
    assert_eq!(pkgs[0].fingerprint, "https://example.com/pkg.tgz");
}

#[test]
fn parse_berry_no_checksum_uses_resolution() {
    let content = "__metadata:\n  version: 6\n\npkg@npm:^1.0.0:\n  version: 1.0.0\n  resolution: \"pkg@npm:1.0.0\"\n  languageName: node\n";
    let pkgs = parse(content);
    assert_eq!(pkgs[0].fingerprint, "pkg@npm:1.0.0");
}

#[test]
fn extract_name_classic_no_at() {
    let result = extract_classic_name("lodash@^4.17.0");
    assert_eq!(result, "lodash");
}

#[test]
fn extract_name_classic_scoped() {
    let result = extract_classic_name("@scope/pkg@^1.0.0");
    assert_eq!(result, "@scope/pkg");
}

#[test]
fn extract_name_classic_no_version() {
    let result = extract_classic_name("bare-name");
    assert_eq!(result, "bare-name");
}

#[test]
fn extract_yarn_name_scoped() {
    let result = extract_yarn_name("@scope/pkg@npm:^1.0.0");
    assert_eq!(result, "@scope/pkg");
}

#[test]
fn extract_yarn_name_no_at() {
    let result = extract_yarn_name("lodash@npm:^4.17.0");
    assert_eq!(result, "lodash");
}

#[test]
fn extract_yarn_name_no_specifier() {
    let result = extract_yarn_name("bare-name");
    assert_eq!(result, "bare-name");
}

#[test]
fn parse_berry_invalid_yaml() {
    let content = "__metadata:\n  version: 6\n{ invalid: yaml: [[[";
    let pkgs = parse(content);
    assert!(pkgs.is_empty());
}
