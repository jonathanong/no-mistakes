use super::*;

const SAMPLE: &str = r#"{
  "lockfileVersion": 0,
  "packages": {
    "lodash": ["lodash@4.17.21", {}, {
      "integrity": "sha512-abc123",
      "resolved": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz"
    }],
    "@scope/pkg": ["@scope/pkg@2.0.0", {}, {
      "integrity": "sha512-scoped"
    }],
    "no-info": ["no-info@1.0.0", {}]
  }
}"#;

#[test]
fn parse_basic() {
    let pkgs = parse(SAMPLE);
    let lodash = pkgs.iter().find(|p| p.name == "lodash").unwrap();
    assert_eq!(lodash.version, "4.17.21");
    assert_eq!(lodash.fingerprint, "sha512-abc123");
    assert_eq!(lodash.kind, ResolutionKind::Registry);
}

#[test]
fn parse_scoped() {
    let pkgs = parse(SAMPLE);
    let scoped = pkgs.iter().find(|p| p.name == "@scope/pkg").unwrap();
    assert_eq!(scoped.version, "2.0.0");
    assert_eq!(scoped.fingerprint, "sha512-scoped");
}

#[test]
fn parse_no_info() {
    let pkgs = parse(SAMPLE);
    let ni = pkgs.iter().find(|p| p.name == "no-info").unwrap();
    assert_eq!(ni.fingerprint, "");
}

#[test]
fn parse_invalid_json() {
    assert!(parse("{ invalid").is_empty());
}

#[test]
fn parse_no_packages() {
    assert!(parse("{}").is_empty());
}

#[test]
fn parse_non_array_entry_skipped() {
    let content = r#"{"packages": {"pkg": "not-an-array"}}"#;
    let pkgs = parse(content);
    assert!(pkgs.is_empty());
}

#[test]
fn parse_uses_resolved_when_no_integrity() {
    let content = r#"{
      "packages": {
        "pkg": ["pkg@1.0.0", {}, {
          "resolved": "https://example.com/pkg.tgz"
        }]
      }
    }"#;
    let pkgs = parse(content);
    assert_eq!(pkgs[0].fingerprint, "https://example.com/pkg.tgz");
}

#[test]
fn parse_empty_specifier_version() {
    let content = r#"{"packages": {"pkg": ["pkg", {}, {"integrity": "sha512-x"}]}}"#;
    let pkgs = parse(content);
    assert_eq!(pkgs[0].version, "");
}

#[test]
fn parse_jsonc_line_comment() {
    let content = "{\n  // lockfileVersion comment\n  \"packages\": {\n    \"pkg\": [\"pkg@1.0.0\", {}, {\"integrity\": \"sha512-x\"}]\n  }\n}";
    let pkgs = parse(content);
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "pkg");
}

#[test]
fn parse_jsonc_block_comment() {
    let content = "{\n  /* a block comment */\n  \"packages\": {\n    \"pkg\": [\"pkg@2.0.0\", {}, {\"integrity\": \"sha512-y\"}]\n  }\n}";
    let pkgs = parse(content);
    assert_eq!(pkgs[0].version, "2.0.0");
}

#[test]
fn parse_jsonc_trailing_comma() {
    let content = "{\n  \"packages\": {\n    \"pkg\": [\"pkg@3.0.0\", {}, {\"integrity\": \"sha512-z\"}],\n  }\n}";
    let pkgs = parse(content);
    assert_eq!(pkgs[0].version, "3.0.0");
}

#[test]
fn strip_jsonc_preserves_slash_in_string() {
    let content = r#"{"packages": {"pkg": ["https://example.com/pkg@1.0.0", {}, {"integrity": "sha512-x"}]}}"#;
    let pkgs = parse(content);
    assert_eq!(pkgs[0].version, "1.0.0");
}

#[test]
fn strip_jsonc_block_comment_preserves_newlines() {
    // Block comment spanning multiple lines — newlines preserved so line numbers stay intact.
    let content = "{\n  /*\n   multi-line\n   comment\n  */\n  \"packages\": {}\n}";
    let pkgs = parse(content);
    assert!(pkgs.is_empty());
}

#[test]
fn parse_real_bun_lock_four_element_tuple() {
    // Real bun.lock format: ["spec", registry_url, peer_deps, {integrity, resolved}]
    // The integrity object is at index 3, not index 2.
    let content = r#"{
      "lockfileVersion": 0,
      "packages": {
        "is-fullwidth-code-point": [
          "is-fullwidth-code-point@3.0.0",
          "",
          {},
          { "integrity": "sha512-zqk+299z" }
        ]
      }
    }"#;
    let pkgs = parse(content);
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "is-fullwidth-code-point");
    assert_eq!(pkgs[0].version, "3.0.0");
    assert_eq!(pkgs[0].fingerprint, "sha512-zqk+299z");
}

#[test]
fn parse_four_element_no_integrity_falls_back_to_resolved() {
    let content = r#"{
      "lockfileVersion": 0,
      "packages": {
        "my-pkg": [
          "my-pkg@1.0.0",
          "",
          {},
          { "resolved": "https://example.com/my-pkg.tgz" }
        ]
      }
    }"#;
    let pkgs = parse(content);
    assert_eq!(pkgs[0].fingerprint, "https://example.com/my-pkg.tgz");
}
