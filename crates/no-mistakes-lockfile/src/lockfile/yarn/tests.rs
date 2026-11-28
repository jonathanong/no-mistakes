use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/lockfile/yarn")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e))
}

#[test]
fn parse_classic_basic() {
    let pkgs = parse(&fixture("classic.lock"));
    let lodash = pkgs.iter().find(|p| p.name == "lodash").unwrap();
    assert_eq!(lodash.version, "4.17.21");
    assert_eq!(lodash.fingerprint, "sha512-abc123");
    assert_eq!(lodash.kind, ResolutionKind::Registry);
}

#[test]
fn parse_classic_scoped() {
    let pkgs = parse(&fixture("classic.lock"));
    let scoped = pkgs.iter().find(|p| p.name == "@scope/pkg").unwrap();
    assert_eq!(scoped.version, "1.0.0");
}

#[test]
fn parse_classic_multiple_specifiers() {
    let pkgs = parse(&fixture("classic.lock"));
    let lodash_entries: Vec<_> = pkgs.iter().filter(|p| p.name == "lodash").collect();
    assert_eq!(
        lodash_entries.len(),
        1,
        "multiple specifiers for same pkg = one entry"
    );
}

#[test]
fn parse_berry_basic() {
    let pkgs = parse(&fixture("berry.lock"));
    let lodash = pkgs.iter().find(|p| p.name == "lodash").unwrap();
    assert_eq!(lodash.version, "4.17.21");
    assert_eq!(lodash.fingerprint, "abc123");
}

#[test]
fn parse_berry_skips_metadata() {
    let pkgs = parse(&fixture("berry.lock"));
    assert!(pkgs.iter().all(|p| p.name != "__metadata"));
}

#[test]
fn parse_berry_scoped() {
    let pkgs = parse(&fixture("berry.lock"));
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

#[test]
fn parse_berry_root_not_mapping() {
    // Valid YAML list, but contains "__metadata:" so berry path is taken;
    // root.as_mapping() returns None → returns empty
    let content = "- __metadata:\n    version: 6\n";
    assert!(parse(content).is_empty());
}

#[test]
fn parse_berry_non_string_key_skipped() {
    // Numeric YAML key (parses as Number, not String) → _ => continue
    let content = "__metadata:\n  version: 6\n\n123:\n  version: 1.0.0\n  checksum: abc\n";
    // Should parse without panic and skip the numeric-keyed entry
    let pkgs = parse(content);
    // Only __metadata (skipped) and numeric key (skipped) — so empty
    assert!(pkgs.is_empty());
}

#[test]
fn extract_yarn_name_scoped_no_version_specifier() {
    // @scope/pkg with no @ after scope → falls through to first.to_string()
    let result = extract_yarn_name("@scope/bare");
    assert_eq!(result, "@scope/bare");
}

#[test]
fn parse_classic_mid_loop_no_integrity() {
    // Two packages; first has only resolved (no integrity).
    // When parser hits second package header, it flushes first via mid-loop path.
    let content = "# yarn lockfile v1\n\npkg1@^1.0.0:\n  version \"1.0.0\"\n  resolved \"https://example.com/pkg1.tgz\"\n\npkg2@^2.0.0:\n  version \"2.0.0\"\n  resolved \"https://example.com/pkg2.tgz\"\n  integrity sha512-abc\n";
    let pkgs = parse(content);
    let p1 = pkgs.iter().find(|p| p.name == "pkg1").unwrap();
    assert_eq!(p1.fingerprint, "https://example.com/pkg1.tgz");
}

#[test]
fn extract_classic_name_scoped_no_version_specifier() {
    // @scope/pkg with no @ after scope in classic header
    let result = extract_classic_name("@scope/bare");
    assert_eq!(result, "@scope/bare");
}
