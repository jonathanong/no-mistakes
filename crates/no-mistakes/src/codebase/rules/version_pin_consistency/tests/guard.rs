use super::extra::PIN;
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

fn leaked_pin(path: &std::path::Path) {
    std::fs::write(
        path,
        "[tools]\n\"aqua:lycheeverse/lychee\" = \"leaked-secret-pin\"\n",
    )
    .unwrap();
}

#[test]
fn mapping_key_line_skips_earlier_substring() {
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        "# node is documented here\npackage:\n  engines:\n    node: 20\n",
        "versions.yml",
        "NODE_VERSION: 20.0.0\n",
    );
    let findings = run(
        tmp.path(),
        &node_yaml("versions.yml"),
        &["versions.yml", PAIR[1]],
    );
    assert_eq!(findings[0].line, 4, "{findings:?}");
    assert!(findings[0].message.contains("invalid pin"), "{findings:?}");
}

#[test]
fn ascii_key_after_multibyte_prefix_does_not_panic() {
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        "package:\n  engines:\n    description: énode\n    node: 20\n",
        "versions.yml",
        "NODE_VERSION: 20.0.0\n",
    );
    let findings = run(
        tmp.path(),
        &node_yaml("versions.yml"),
        &["versions.yml", PAIR[1]],
    );
    assert_eq!(findings[0].line, 4, "{findings:?}");
    assert!(findings[0].message.contains("invalid pin"), "{findings:?}");
}

#[test]
fn disable_next_line_targets_the_mapping_key() {
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        "# node is documented here\npackage:\n  engines:\n    # no-mistakes-disable-next-line version-pin-consistency\n    node: 20\n",
        "versions.yml",
        "NODE_VERSION: 20.0.0\n",
    );
    assert!(run(
        tmp.path(),
        &node_yaml("versions.yml"),
        &["versions.yml", PAIR[1]]
    )
    .is_empty());
}

#[test]
fn later_stale_capture_is_still_a_finding() {
    let tmp = tmp_pair(
        PIN,
        "# example LYCHEE_VERSION: 0.24.2\nLYCHEE_VERSION: 0.24.1\n",
    );
    let findings = run(tmp.path(), OPTIONS, &PAIR);
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
    let tmp = tmp_pair(
        PIN,
        "LYCHEE_VERSION: 0.24.2\n# also LYCHEE_VERSION: 0.24.2\n",
    );
    assert!(run(tmp.path(), OPTIONS, &PAIR).is_empty());
}

#[test]
fn absolute_source_file_does_not_leak_external_contents() {
    let outside = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    leaked_pin(outside.path());
    let tmp = tmp_pair(PIN, "LYCHEE_VERSION: 0.24.1\n");
    let yaml = OPTIONS.replace(
        "sourceFile: .mise.toml",
        &format!("sourceFile: {}", outside.path().display()),
    );
    assert_no_leak(&run(tmp.path(), &yaml, &PAIR));
}

#[test]
fn parent_dir_source_file_does_not_leak_external_contents() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    leaked_pin(&parent.path().join("outside.toml"));
    write_pair(&root, PIN, ".mise.toml", "LYCHEE_VERSION: 0.24.1\n");
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: ../outside.toml");
    assert_no_leak(&run(&root, &yaml, &PAIR));
}

#[cfg(unix)]
#[test]
fn symlink_source_outside_repo_is_not_read() {
    let outside = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    leaked_pin(outside.path());
    let tmp = tmp_pair(PIN, "LYCHEE_VERSION: 0.24.1\n");
    std::os::unix::fs::symlink(outside.path(), tmp.path().join("pin.toml")).unwrap();
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: pin.toml");
    assert_no_leak(&run(tmp.path(), &yaml, &["pin.toml", PAIR[1]]));
}

#[cfg(unix)]
#[test]
fn directory_symlink_escape_is_not_read() {
    let outside = tempfile::tempdir().unwrap();
    leaked_pin(&outside.path().join("pin.toml"));
    let tmp = tmp_pair(PIN, "LYCHEE_VERSION: 0.24.1\n");
    std::os::unix::fs::symlink(outside.path(), tmp.path().join("escape")).unwrap();
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: escape/pin.toml");
    assert_no_leak(&run(tmp.path(), &yaml, &["escape/pin.toml", PAIR[1]]));
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
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        "tools:\n  \"foo.bar\": 8\n",
        "versions.yml",
        "TOOL_VERSION: 1.2.3\n",
    );
    let findings = run(
        tmp.path(),
        dotted_remainder_yaml(),
        &["versions.yml", PAIR[1]],
    );
    assert_eq!(findings[0].line, 2, "{findings:?}");
    assert!(findings[0].message.contains("invalid pin"), "{findings:?}");
}

#[test]
fn disable_next_line_targets_literal_dotted_key() {
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        "tools:\n  # no-mistakes-disable-next-line version-pin-consistency\n  \"foo.bar\": 8\n",
        "versions.yml",
        "TOOL_VERSION: 1.2.3\n",
    );
    assert!(run(
        tmp.path(),
        dotted_remainder_yaml(),
        &["versions.yml", PAIR[1]]
    )
    .is_empty());
}
