use super::*;

fn node_yaml(source_file: &str) -> String {
    format!(
        r#"
sourceFile: {source_file}
sourceKey: package.engines.node
anchors:
  - file: {}
    pattern: 'NODE_VERSION:\s*(\d+\.\d+\.\d+)'
    label: node
"#,
        PAIR[1]
    )
}

fn assert_no_leak(findings: &[RuleFinding]) {
    let blob = format!("{findings:?}");
    assert!(
        !blob.contains("leaked-secret-pin"),
        "external pin leaked: {blob}"
    );
}

#[test]
fn mapping_key_line_skips_earlier_substring() {
    let root = fixture("mapping-key-line");
    let findings = run(
        &root,
        &node_yaml("versions.yml"),
        &["versions.yml", PAIR[1]],
    );
    assert_eq!(findings[0].line, 4, "{findings:?}");
    assert!(findings[0].message.contains("invalid pin"), "{findings:?}");
}

#[test]
fn ascii_key_after_multibyte_prefix_does_not_panic() {
    let root = fixture("multibyte-key");
    let findings = run(
        &root,
        &node_yaml("versions.yml"),
        &["versions.yml", PAIR[1]],
    );
    assert_eq!(findings[0].line, 4, "{findings:?}");
    assert!(findings[0].message.contains("invalid pin"), "{findings:?}");
}

#[test]
fn disable_next_line_targets_the_mapping_key() {
    let root = fixture("disable-mapping-key");
    assert!(run(
        &root,
        &node_yaml("versions.yml"),
        &["versions.yml", PAIR[1]]
    )
    .is_empty());
}

#[test]
fn later_stale_capture_is_still_a_finding() {
    let root = fixture("later-stale-capture");
    let findings = run(&root, OPTIONS, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("0.24.1")
                && finding.message.contains("version mismatch")),
        "{findings:?}"
    );
}

#[test]
fn all_matching_captures_pass() {
    let root = fixture("all-matching-captures");
    assert!(run(&root, OPTIONS, &PAIR).is_empty());
}

#[test]
fn absolute_source_file_does_not_leak_external_contents() {
    let root = fixture("external-pin/repo");
    let outside = fixture("external-pin").join("outside.toml");
    let yaml = OPTIONS.replace(
        "sourceFile: .mise.toml",
        &format!("sourceFile: {}", outside.display()),
    );
    assert_no_leak(&run(&root, &yaml, &PAIR));
}

#[test]
fn parent_dir_source_file_does_not_leak_external_contents() {
    let root = fixture("external-pin/repo");
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: ../outside.toml");
    assert_no_leak(&run(&root, &yaml, &PAIR));
}

#[cfg(unix)]
#[test]
fn symlink_source_outside_repo_is_not_read() {
    let root = fixture("symlink-pin/repo");
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: pin.toml");
    assert_no_leak(&run(&root, &yaml, &["pin.toml", PAIR[1]]));
}

#[cfg(unix)]
#[test]
fn directory_symlink_escape_is_not_read() {
    let root = fixture("dir-symlink/repo");
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: escape/pin.toml");
    assert_no_leak(&run(&root, &yaml, &["escape/pin.toml", PAIR[1]]));
}

fn dotted_remainder_yaml() -> &'static str {
    r#"
sourceFile: versions.yml
sourceKey: tools.foo.bar
anchors:
  - file: .github/actions/setup-lychee/action.yml
    pattern: 'TOOL_VERSION:\s*(\d+\.\d+\.\d+)'
    label: tool
"#
}

#[test]
fn literal_dotted_remainder_is_the_mapping_key_line() {
    let root = fixture("literal-dotted");
    let findings = run(&root, dotted_remainder_yaml(), &["versions.yml", PAIR[1]]);
    assert_eq!(findings[0].line, 2, "{findings:?}");
    assert!(findings[0].message.contains("invalid pin"), "{findings:?}");
}

#[test]
fn disable_next_line_targets_literal_dotted_key() {
    let root = fixture("disable-dotted-key");
    assert!(run(&root, dotted_remainder_yaml(), &["versions.yml", PAIR[1]]).is_empty());
}
