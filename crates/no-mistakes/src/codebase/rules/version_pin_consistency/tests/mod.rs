use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

const OPTIONS: &str = r#"
sourceFile: .mise.toml
sourceKey: tools.aqua:lycheeverse/lychee
anchors:
  - file: .github/actions/setup-lychee/action.yml
    pattern: 'LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)'
    label: lychee
"#;

const PAIR: [&str; 2] = [".mise.toml", ".github/actions/setup-lychee/action.yml"];

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/version-pin-consistency/fixture")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    config_filtered(yaml, &[], &[])
}

fn config_filtered(yaml: &str, include: &[&str], exclude: &[&str]) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            include: include.iter().map(|path| path.to_string()).collect(),
            exclude: exclude.iter().map(|path| path.to_string()).collect(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path, yaml: &str, files: &[&str]) -> Vec<RuleFinding> {
    run_config(root, &config(yaml), files)
}

fn run_config(root: &Path, cfg: &NoMistakesConfig, files: &[&str]) -> Vec<RuleFinding> {
    let files: Vec<PathBuf> = files.iter().map(|file| root.join(file)).collect();
    check_with_files(root, cfg, &files).unwrap()
}

fn write_pair(root: &Path, source: &str, source_name: &str, anchor: &str) {
    let action = root.join(".github/actions/setup-lychee/action.yml");
    std::fs::create_dir_all(action.parent().unwrap()).unwrap();
    std::fs::write(root.join(source_name), source).unwrap();
    std::fs::write(action, anchor).unwrap();
}

fn tmp_pair(source: &str, anchor: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write_pair(tmp.path(), source, ".mise.toml", anchor);
    tmp
}

#[test]
fn matching_pins_pass() {
    let root = fixture("pass");
    assert!(run(&root, OPTIONS, &PAIR).is_empty());
}

#[test]
fn mismatch_is_a_finding() {
    let findings = run(&fixture("fail"), OPTIONS, &PAIR);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("version mismatch"),
        "{findings:?}"
    );
    assert!(findings[0].file.contains("action.yml"), "{findings:?}");
    assert!(findings[0].line > 1, "{findings:?}");
}

#[test]
fn skip_when_configured_files_are_not_tracked() {
    let findings = run(&fixture("pass"), OPTIONS, &["README.md"]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn skip_absent_fixture_is_silent() {
    let findings = run(&fixture("skip-absent"), OPTIONS, &["notes.txt"]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn empty_options_are_silent() {
    let root = fixture("pass");
    assert!(run(&root, "{}", &[".mise.toml"]).is_empty());
}

#[test]
fn missing_source_key_is_a_finding() {
    let tmp = tmp_pair("[tools]\nfoo = \"1.0.0\"\n", "LYCHEE_VERSION: 0.24.2\n");
    let findings = run(tmp.path(), OPTIONS, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("not found")),
        "{findings:?}"
    );
}

#[test]
fn non_string_pin_is_a_finding() {
    let tmp = tmp_pair(
        "[tools]\n\"aqua:lycheeverse/lychee\" = 8\n",
        "LYCHEE_VERSION: 0.24.2\n",
    );
    let findings = run(tmp.path(), OPTIONS, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("invalid pin")),
        "{findings:?}"
    );
}

#[test]
fn missing_capture_is_a_finding() {
    let tmp = tmp_pair(
        "[tools]\n\"aqua:lycheeverse/lychee\" = \"0.24.2\"\n",
        "# no version here\n",
    );
    let findings = run(tmp.path(), OPTIONS, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("could not find")),
        "{findings:?}"
    );
}

#[test]
fn invalid_toml_is_a_finding() {
    let tmp = tmp_pair("[tools]\n\"broken", "LYCHEE_VERSION: 0.24.2\n");
    let findings = run(tmp.path(), OPTIONS, &PAIR);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("failed to parse TOML")),
        "{findings:?}"
    );
}

#[test]
fn json_source_file_is_supported() {
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        r#"{"tools":{"aqua:lycheeverse/lychee":"0.24.2"}}"#,
        "versions.json",
        "LYCHEE_VERSION: 0.24.2\n",
    );
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: versions.json");
    assert!(run(tmp.path(), &yaml, &["versions.json", PAIR[1]]).is_empty());
}

#[test]
fn yaml_source_file_is_supported() {
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        "tools:\n  \"aqua:lycheeverse/lychee\": \"0.24.2\"\n",
        "versions.yml",
        "LYCHEE_VERSION: 0.24.2\n",
    );
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: versions.yml");
    assert!(run(tmp.path(), &yaml, &["versions.yml", PAIR[1]]).is_empty());
}

#[test]
fn nested_dotted_key_is_walked() {
    let tmp = tempfile::tempdir().unwrap();
    write_pair(
        tmp.path(),
        "{\"package\":{\"engines\":{\"node\":\"20.0.0\"}}}",
        "package.json",
        "NODE_VERSION: 20.0.0\n",
    );
    let yaml = r#"
sourceFile: package.json
sourceKey: package.engines.node
anchors:
  - file: .github/actions/setup-lychee/action.yml
    pattern: 'NODE_VERSION:\s*(\d+\.\d+\.\d+)'
    label: node
"#;
    assert!(run(tmp.path(), yaml, &["package.json", PAIR[1]]).is_empty());
}

#[test]
fn pattern_must_have_one_capturing_group() {
    let tmp = tmp_pair(
        "[tools]\n\"aqua:lycheeverse/lychee\" = \"0.24.2\"\n",
        "LYCHEE_VERSION: 0.24.2\n",
    );
    let yaml = OPTIONS.replace(
        r"LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)",
        r"LYCHEE_VERSION:\s*\d+\.\d+\.\d+",
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
fn runs_when_only_the_anchor_is_tracked() {
    let findings = run(&fixture("fail"), OPTIONS, &[PAIR[1]]);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("version mismatch")),
        "{findings:?}"
    );
}

#[test]
fn does_not_report_untracked_anchors_when_only_source_is_listed() {
    let findings = run(&fixture("fail"), OPTIONS, &[".mise.toml"]);
    assert!(findings.is_empty(), "{findings:?}");
}

mod extra;
mod guard;
