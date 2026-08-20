use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-test-timeout-literals/fixture")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn workflow_tests(root: &Path) -> Vec<PathBuf> {
    [
        root.join(".github/workflows/ci.test.mts"),
        root.join(".github/workflows/ci.test.ts"),
    ]
    .into_iter()
    .filter(|path| path.exists())
    .collect()
}

fn run(root: &Path, yaml: &str) -> Vec<RuleFinding> {
    check_with_files(root, &config(yaml), &workflow_tests(root)).unwrap()
}

fn scan_source(source: &str, yaml: &str) -> Vec<RuleFinding> {
    scan::check_source(
        ".github/workflows/ci.test.mts",
        source,
        &compile_options(serde_yaml::from_str(yaml).unwrap()),
    )
}

#[test]
fn yaml_fragment_is_a_finding() {
    let findings = run(&fixture("fail"), "{}");
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("duplicates a timeout-minutes value")
            && finding.target.as_deref()
                == Some("expect(workflowSource).toContain('timeout-minutes: 15')")),
        "{findings:?}"
    );
}

#[test]
fn property_to_be_is_a_finding() {
    let findings = scan_source("expect(step?.['timeout-minutes']).toBe(10)\n", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("duplicates a timeout-minutes value"));
}

#[test]
fn property_to_equal_is_a_finding() {
    let findings = scan_source("expect(job.timeoutMinutes).toEqual(8)\n", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn to_contain_with_digit_is_a_finding() {
    let findings = scan_source(
        r#"expect(job?.['timeout-minutes']).toContain("&& '45'")"#,
        "{}",
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn range_assertion_passes() {
    assert!(scan_source(
        "expect(step?.['timeout-minutes']).toBeLessThanOrEqual(job?.['timeout-minutes'])\n",
        "{}"
    )
    .is_empty());
}

#[test]
fn to_contain_without_digit_passes() {
    assert!(scan_source(
        "expect(job?.['timeout-minutes']).toContain('HOST_SUPERVISION_READY')\n",
        "{}"
    )
    .is_empty());
}

#[test]
fn fixture_fail_covers_each_shape() {
    let findings = run(&fixture("fail"), "{}");
    assert_eq!(findings.len(), 4, "{findings:?}");
}

#[test]
fn fixture_pass_is_silent() {
    let root = fixture("pass");
    let mut files = workflow_tests(&root);
    files.push(root.join("src/other.test.mts"));
    let findings = check_with_files(&root, &config("{}"), &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn allow_entry_skips_the_matching_line() {
    assert!(run(
        &fixture("allow"),
        r#"
allow:
  - file: .github/workflows/ci.test.mts
    text: "timeout-minutes: 20"
    reason: pins fromJSON branch values the timeout rule cannot resolve
"#
    )
    .is_empty());
}

#[test]
fn empty_reason_is_a_finding() {
    let findings = scan_source(
        "timeout-minutes: 20\n",
        r#"
allow:
  - file: .github/workflows/ci.test.mts
    text: "timeout-minutes: 20"
    reason: " "
"#,
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("has no reason"));
}

#[test]
fn stale_allow_entry_is_a_finding() {
    let findings = run(
        &fixture("pass"),
        r#"
allow:
  - file: .github/workflows/ci.test.mts
    text: "timeout-minutes: 99"
    reason: leftover pin
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("stale")),
        "{findings:?}"
    );
}

#[test]
fn allow_for_unscanned_file_is_not_stale() {
    let findings = run(
        &fixture("pass"),
        r#"
allow:
  - file: .github/workflows/missing.test.mts
    text: "timeout-minutes: 20"
    reason: other file
"#,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn custom_include_still_scans_workflow_tests() {
    assert!(run(
        &fixture("pass"),
        "include: [\".github/workflows/ci.test.mts\"]\n"
    )
    .is_empty());
}

#[test]
fn include_without_matches_is_silent() {
    assert!(run(&fixture("fail"), "include: [\"nope.test.mts\"]").is_empty());
}

#[test]
fn disable_file_comment_skips_the_test() {
    assert!(scan_source(
        "// no-mistakes-disable-file github-actions-test-timeout-literals\nexpect(step?.['timeout-minutes']).toBe(10)\n",
        "{}"
    )
    .is_empty());
}

#[test]
fn unreadable_test_is_skipped() {
    let root = fixture("pass");
    let path = root.join(".github/workflows/missing.test.mts");
    let sources = super::super::source_store_for_files(&[]);
    let opts = compile_options(Options::default());
    assert!(scan::check_file(&root, &path, &opts, &sources).is_empty());
}

#[test]
fn yaml_without_digits_is_silent() {
    assert!(scan_source("timeout-minutes: fromJSON(vars.BUDGET)\n", "{}").is_empty());
}

#[test]
fn allow_entry_default_is_empty() {
    let allow = AllowEntry::default();
    assert_eq!(allow.clone().file, allow.file);
    assert!(allow.text.is_empty());
    assert!(allow.reason.is_empty());
}
