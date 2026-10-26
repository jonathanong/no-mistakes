use super::*;

fn extra_yaml(source_file: &str, extra: &str) -> String {
    format!(
        "sourceFile: {source_file}\nsourceKey: tools.aqua:lycheeverse/lychee\nanchors:\n{extra}"
    )
}

fn default_anchor(pattern: &str, label: &str) -> String {
    let label_line = if label.is_empty() {
        String::new()
    } else {
        format!("    label: {label}\n")
    };
    format!(
        "  - file: {}\n    pattern: '{pattern}'\n{label_line}",
        PAIR[1]
    )
}

#[test]
fn include_keeps_only_remaining_anchors() {
    let findings = run_config(
        &fixture("fail"),
        &config_filtered(OPTIONS, &[".github/**"], &[]),
        &PAIR,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("version mismatch")),
        "{findings:?}"
    );
}

#[test]
fn exclude_drops_anchor_findings() {
    let findings = run_config(
        &fixture("fail"),
        &config_filtered(OPTIONS, &[], &[".github/**"]),
        &PAIR,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn leading_dot_slash_paths_match_tracked_rels() {
    let yaml = OPTIONS
        .replace("sourceFile: .mise.toml", "sourceFile: ./.mise.toml")
        .replace(
            "file: .github/actions/setup-lychee/action.yml",
            "file: ./.github/actions/setup-lychee/action.yml",
        );
    let findings = run(&fixture("fail"), &yaml, &PAIR);
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn disable_file_comment_suppresses_locally() {
    let root = fixture("disable-file");
    assert!(run(&root, OPTIONS, &PAIR).is_empty());
}

#[test]
fn deferred_suppression_still_emits_disabled_file() {
    let root = fixture("disable-file");
    let files: Vec<PathBuf> = PAIR.iter().map(|file| root.join(file)).collect();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let findings = check_with_files_sources_and_deferred_suppression(
        &root,
        &config(OPTIONS),
        &files,
        &sources,
        true,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("version mismatch"),
        "{findings:?}"
    );
}

#[test]
fn invalid_pattern_is_a_finding() {
    let root = fixture("matching-short-anchor");
    let yaml = OPTIONS.replace(r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)", r"LYCHEE_VERSION:(");
    let findings = run(&root, &yaml, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("pattern is invalid")),
        "{findings:?}"
    );
}

#[test]
fn two_capturing_groups_are_rejected() {
    let root = fixture("matching-short-anchor");
    let yaml = OPTIONS.replace(
        r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)",
        r"LYCHEE_VERSION:\s*(\d+)\.(\d+\.\d+)",
    );
    let findings = run(&root, &yaml, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("exactly one capturing group")),
        "{findings:?}"
    );
}

#[test]
fn empty_anchor_file_is_skipped() {
    let root = fixture("pin-mismatch");
    let yaml = extra_yaml(
        ".mise.toml",
        "  - file: \"\"\n    pattern: '(x)'\n    label: empty\n",
    );
    assert!(run(&root, &yaml, &PAIR).is_empty());
}

#[test]
fn empty_label_uses_the_anchor_path() {
    let root = fixture("pin-mismatch");
    let yaml = extra_yaml(
        ".mise.toml",
        &default_anchor(r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)", ""),
    );
    let findings = run(&root, &yaml, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains(PAIR[1])),
        "{findings:?}"
    );
}

#[test]
fn pin_kinds_are_reported() {
    for (name, source_file, kind) in [
        ("pin-kind-true", ".mise.toml", "true"),
        ("pin-kind-null", "versions.yml", "null"),
        ("pin-kind-array", "versions.yml", "array"),
        ("pin-kind-object", "versions.yml", "object"),
    ] {
        let root = fixture(name);
        let yaml = OPTIONS.replace(
            "sourceFile: .mise.toml",
            &format!("sourceFile: {source_file}"),
        );
        let findings = run(&root, &yaml, &[source_file, PAIR[1]]);
        assert!(
            findings
                .iter()
                .any(|finding| finding.message.contains(kind)),
            "{kind} {findings:?}"
        );
    }
}

#[test]
fn undotted_source_key_is_resolved() {
    let root = fixture("undotted-scalar");
    let yaml = r#"
sourceFile: version.yml
sourceKey: ignored
anchors:
  - file: .github/actions/setup-lychee/action.yml
    pattern: 'TOOL_VERSION:\s*(\d+\.\d+\.\d+)'
    label: tool
"#;
    // A YAML scalar document is not a mapping, so the undotted key is missing.
    let findings = run(&root, yaml, &["version.yml", PAIR[1]]);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("not found")),
        "{findings:?}"
    );
}

#[test]
fn empty_toml_source_is_a_missing_key() {
    let root = fixture("empty-toml");
    let findings = run(&root, OPTIONS, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("not found")),
        "{findings:?}"
    );
}

#[test]
fn invalid_yaml_source_is_a_finding() {
    let root = fixture("invalid-yaml");
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: versions.yml");
    let findings = run(&root, &yaml, &["versions.yml", PAIR[1]]);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("failed to parse YAML")),
        "{findings:?}"
    );
}

#[test]
fn jsonc_source_file_is_supported() {
    let root = fixture("jsonc-source");
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: versions.jsonc");
    assert!(run(&root, &yaml, &["versions.jsonc", PAIR[1]]).is_empty());
}

#[test]
fn tagged_yaml_pin_is_invalid() {
    let root = fixture("tagged-yaml");
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: versions.yml");
    let findings = run(&root, &yaml, &["versions.yml", PAIR[1]]);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("invalid pin")),
        "{findings:?}"
    );
}

#[test]
fn optional_capture_without_a_match_is_a_mismatch() {
    let root = fixture("optional-capture");
    let yaml = OPTIONS.replace(
        r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)",
        r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)?",
    );
    let findings = run(&root, &yaml, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("version mismatch")),
        "{findings:?}"
    );
}

#[test]
fn missing_key_line_defaults_when_needle_is_absent() {
    let root = fixture("missing-key-line");
    let findings = run(&root, OPTIONS, &PAIR);
    assert_eq!(findings[0].line, 1, "{findings:?}");
}

#[test]
fn undotted_mapping_key_matches() {
    let root = fixture("undotted-mapping");
    let yaml = r#"
sourceFile: version.yml
sourceKey: pin
anchors:
  - file: .github/actions/setup-lychee/action.yml
    pattern: 'TOOL_VERSION:\s*(\d+\.\d+\.\d+)'
    label: tool
"#;
    assert!(run(&root, yaml, &["version.yml", PAIR[1]]).is_empty());
}

#[test]
fn empty_files_after_filter_are_silent() {
    let findings = run_config(
        &fixture("fail"),
        &config_filtered(OPTIONS, &["does-not-exist/**"], &[]),
        &PAIR,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn invalid_pattern_on_untracked_source_is_silent() {
    let root = fixture("matching-short-anchor");
    let yaml = OPTIONS.replace(r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)", r"LYCHEE_VERSION:(");
    let findings = run(&root, &yaml, &[PAIR[1]]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn missing_key_on_untracked_source_is_silent() {
    let root = fixture("missing-source-key");
    let findings = run(&root, OPTIONS, &[PAIR[1]]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn non_string_pin_on_untracked_source_is_silent() {
    let root = fixture("non-string-pin");
    let findings = run(&root, OPTIONS, &[PAIR[1]]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn empty_source_file_option_is_silent() {
    let yaml = extra_yaml(
        "\"\"",
        &default_anchor(r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)", "lychee"),
    );
    assert!(run(&fixture("fail"), &yaml, &PAIR).is_empty());
}

#[test]
fn unicode_value_before_mapping_key_does_not_panic() {
    let root = fixture("unicode-key");
    let yaml = r#"
sourceFile: versions.yml
sourceKey: 版本
anchors:
  - file: .github/actions/setup-lychee/action.yml
    pattern: 'TOOL_VERSION:\s*(\d+\.\d+\.\d+)'
    label: tool
"#;
    let findings = run(&root, yaml, &["versions.yml", PAIR[1]]);
    assert_eq!(findings[0].line, 3, "{findings:?}");
    assert!(findings[0].message.contains("invalid pin"), "{findings:?}");
}

#[test]
fn tracked_source_without_anchors_reports_missing_key() {
    let root = fixture("missing-source-key");
    let findings = run(&root, OPTIONS, &[".mise.toml"]);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("not found")),
        "{findings:?}"
    );
}

#[test]
fn tracked_source_without_anchors_reports_non_string_pin() {
    let root = fixture("non-string-pin");
    let findings = run(&root, OPTIONS, &[".mise.toml"]);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("invalid pin")),
        "{findings:?}"
    );
}

#[test]
fn exclude_all_anchors_still_validates_source_pin() {
    let root = fixture("missing-source-key");
    let findings = run_config(
        &root,
        &config_filtered(OPTIONS, &[], &[".github/**"]),
        &PAIR,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("not found")),
        "{findings:?}"
    );
}
