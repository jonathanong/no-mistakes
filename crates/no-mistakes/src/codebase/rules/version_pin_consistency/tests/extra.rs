use super::*;

pub(super) const PIN: &str = "[tools]\n\"aqua:lycheeverse/lychee\" = \"0.24.2\"\n";

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
    let tmp = tmp_pair(
        PIN,
        "# no-mistakes-disable-file version-pin-consistency\nLYCHEE_VERSION: 0.24.1\n",
    );
    assert!(run(tmp.path(), OPTIONS, &PAIR).is_empty());
}

#[test]
fn deferred_suppression_still_emits_disabled_file() {
    let tmp = tmp_pair(
        PIN,
        "# no-mistakes-disable-file version-pin-consistency\nLYCHEE_VERSION: 0.24.1\n",
    );
    let files: Vec<PathBuf> = PAIR.iter().map(|file| tmp.path().join(file)).collect();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let findings = check_with_files_sources_and_deferred_suppression(
        tmp.path(),
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
    let tmp = tmp_pair(PIN, "LYCHEE_VERSION: 0.24.2\n");
    let yaml = OPTIONS.replace(r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)", r"LYCHEE_VERSION:(");
    let findings = run(tmp.path(), &yaml, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("pattern is invalid")),
        "{findings:?}"
    );
}

#[test]
fn two_capturing_groups_are_rejected() {
    let tmp = tmp_pair(PIN, "LYCHEE_VERSION: 0.24.2\n");
    let yaml = OPTIONS.replace(
        r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)",
        r"LYCHEE_VERSION:\s*(\d+)\.(\d+\.\d+)",
    );
    let findings = run(tmp.path(), &yaml, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("exactly one capturing group")),
        "{findings:?}"
    );
}

#[test]
fn empty_anchor_file_is_skipped() {
    let tmp = tmp_pair(PIN, "LYCHEE_VERSION: 0.24.1\n");
    let yaml = extra_yaml(
        ".mise.toml",
        "  - file: \"\"\n    pattern: '(x)'\n    label: empty\n",
    );
    assert!(run(tmp.path(), &yaml, &PAIR).is_empty());
}

#[test]
fn empty_label_uses_the_anchor_path() {
    let tmp = tmp_pair(PIN, "LYCHEE_VERSION: 0.24.1\n");
    let yaml = extra_yaml(
        ".mise.toml",
        &default_anchor(r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)", ""),
    );
    let findings = run(tmp.path(), &yaml, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains(PAIR[1])),
        "{findings:?}"
    );
}

#[test]
fn pin_kinds_are_reported() {
    for (source, kind) in [
        ("[tools]\n\"aqua:lycheeverse/lychee\" = true\n", "true"),
        ("tools:\n  \"aqua:lycheeverse/lychee\": null\n", "null"),
        ("tools:\n  \"aqua:lycheeverse/lychee\": [1]\n", "array"),
        ("tools:\n  \"aqua:lycheeverse/lychee\": {a: 1}\n", "object"),
    ] {
        let tmp = tempfile::tempdir().unwrap();
        let name = if source.starts_with("tools:") {
            "versions.yml"
        } else {
            ".mise.toml"
        };
        write_pair(tmp.path(), source, name, "LYCHEE_VERSION: 0.24.2\n");
        let yaml = OPTIONS.replace("sourceFile: .mise.toml", &format!("sourceFile: {name}"));
        let findings = run(tmp.path(), &yaml, &[name, PAIR[1]]);
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
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        "\"1.2.3\"\n",
        "version.yml",
        "TOOL_VERSION: 1.2.3\n",
    );
    let yaml = r#"
sourceFile: version.yml
sourceKey: ignored
anchors:
  - file: .github/actions/setup-lychee/action.yml
    pattern: 'TOOL_VERSION:\s*(\d+\.\d+\.\d+)'
    label: tool
"#;
    // A YAML scalar document is not a mapping, so the undotted key is missing.
    let findings = run(tmp.path(), yaml, &["version.yml", PAIR[1]]);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("not found")),
        "{findings:?}"
    );
}

#[test]
fn empty_toml_source_is_a_missing_key() {
    let tmp = tmp_pair("", "LYCHEE_VERSION: 0.24.2\n");
    let findings = run(tmp.path(), OPTIONS, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("not found")),
        "{findings:?}"
    );
}

#[test]
fn invalid_yaml_source_is_a_finding() {
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        ":\n",
        "versions.yml",
        "LYCHEE_VERSION: 0.24.2\n",
    );
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: versions.yml");
    let findings = run(tmp.path(), &yaml, &["versions.yml", PAIR[1]]);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("failed to parse YAML")),
        "{findings:?}"
    );
}

#[test]
fn jsonc_source_file_is_supported() {
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        "{\n  // pin\n  \"tools\": { \"aqua:lycheeverse/lychee\": \"0.24.2\" }\n}\n",
        "versions.jsonc",
        "LYCHEE_VERSION: 0.24.2\n",
    );
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: versions.jsonc");
    assert!(run(tmp.path(), &yaml, &["versions.jsonc", PAIR[1]]).is_empty());
}

#[test]
fn tagged_yaml_pin_is_invalid() {
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        "tools:\n  \"aqua:lycheeverse/lychee\": !custom \"hello\"\n",
        "versions.yml",
        "LYCHEE_VERSION: 0.24.2\n",
    );
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: versions.yml");
    let findings = run(tmp.path(), &yaml, &["versions.yml", PAIR[1]]);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("invalid pin")),
        "{findings:?}"
    );
}

#[test]
fn untracked_source_parse_error_is_silent() {
    let tmp = tmp_pair("[tools]\n\"broken", "LYCHEE_VERSION: 0.24.2\n");
    let findings = run(tmp.path(), OPTIONS, &[PAIR[1]]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn optional_capture_without_a_match_is_a_mismatch() {
    let tmp = tmp_pair(PIN, "LYCHEE_VERSION:\n");
    let yaml = OPTIONS.replace(
        r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)",
        r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)?",
    );
    let findings = run(tmp.path(), &yaml, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("version mismatch")),
        "{findings:?}"
    );
}

#[test]
fn missing_key_line_defaults_when_needle_is_absent() {
    let tmp = tmp_pair("[other]\nfoo = \"1\"\n", "LYCHEE_VERSION: 0.24.2\n");
    let findings = run(tmp.path(), OPTIONS, &PAIR);
    assert_eq!(findings[0].line, 1, "{findings:?}");
}

#[test]
fn undotted_mapping_key_matches() {
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        "pin: \"1.2.3\"\n",
        "version.yml",
        "TOOL_VERSION: 1.2.3\n",
    );
    let yaml = r#"
sourceFile: version.yml
sourceKey: pin
anchors:
  - file: .github/actions/setup-lychee/action.yml
    pattern: 'TOOL_VERSION:\s*(\d+\.\d+\.\d+)'
    label: tool
"#;
    assert!(run(tmp.path(), yaml, &["version.yml", PAIR[1]]).is_empty());
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
    let tmp = tmp_pair(PIN, "LYCHEE_VERSION: 0.24.2\n");
    let yaml = OPTIONS.replace(r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)", r"LYCHEE_VERSION:(");
    let findings = run(tmp.path(), &yaml, &[PAIR[1]]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn missing_key_on_untracked_source_is_silent() {
    let tmp = tmp_pair("[tools]\nfoo = \"1.0.0\"\n", "LYCHEE_VERSION: 0.24.2\n");
    let findings = run(tmp.path(), OPTIONS, &[PAIR[1]]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn non_string_pin_on_untracked_source_is_silent() {
    let tmp = tmp_pair(
        "[tools]\n\"aqua:lycheeverse/lychee\" = 8\n",
        "LYCHEE_VERSION: 0.24.2\n",
    );
    let findings = run(tmp.path(), OPTIONS, &[PAIR[1]]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn missing_source_on_disk_with_tracked_anchor_is_silent() {
    let tmp = tempfile::tempdir().unwrap();
    let action = tmp.path().join(PAIR[1]);
    std::fs::create_dir_all(action.parent().unwrap()).unwrap();
    std::fs::write(&action, "LYCHEE_VERSION: 0.24.2\n").unwrap();
    let findings = run(tmp.path(), OPTIONS, &[PAIR[1]]);
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
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        "description: 版本\nnote: \"版本\"\n版本: 123\n",
        "versions.yml",
        "TOOL_VERSION: 1.2.3\n",
    );
    let yaml = r#"
sourceFile: versions.yml
sourceKey: 版本
anchors:
  - file: .github/actions/setup-lychee/action.yml
    pattern: 'TOOL_VERSION:\s*(\d+\.\d+\.\d+)'
    label: tool
"#;
    let findings = run(tmp.path(), yaml, &["versions.yml", PAIR[1]]);
    assert_eq!(findings[0].line, 3, "{findings:?}");
    assert!(findings[0].message.contains("invalid pin"), "{findings:?}");
}

#[test]
fn tracked_source_without_anchors_reports_missing_key() {
    let tmp = tmp_pair("[tools]\nfoo = \"1.0.0\"\n", "LYCHEE_VERSION: 0.24.2\n");
    let findings = run(tmp.path(), OPTIONS, &[".mise.toml"]);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("not found")),
        "{findings:?}"
    );
}

#[test]
fn tracked_source_without_anchors_reports_non_string_pin() {
    let tmp = tmp_pair(
        "[tools]\n\"aqua:lycheeverse/lychee\" = 8\n",
        "LYCHEE_VERSION: 0.24.2\n",
    );
    let findings = run(tmp.path(), OPTIONS, &[".mise.toml"]);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("invalid pin")),
        "{findings:?}"
    );
}

#[test]
fn exclude_all_anchors_still_validates_source_pin() {
    let tmp = tmp_pair("[tools]\nfoo = \"1.0.0\"\n", "LYCHEE_VERSION: 0.24.2\n");
    let findings = run_config(
        tmp.path(),
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
