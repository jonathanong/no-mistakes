use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn check_file(
    path: &Path,
    root: &Path,
    exclude_set: &GlobSet,
    uses_re: &Regex,
    sha_re: &Regex,
    version_re: &Regex,
) -> Vec<RuleFinding> {
    let sources = crate::codebase::rules::source_store_for_files(&[path.to_path_buf()]);
    check_file_with_sources(
        path,
        root,
        exclude_set,
        uses_re,
        sha_re,
        version_re,
        &sources,
    )
}

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-pinned-hash/fixture")
        .join(path)
}

fn config_with_options(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn case_root(name: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("github-actions-pinned-hash")
            .join("cases")
            .join(name),
    )
}

fn run_on_workflow(name: &str) -> Vec<RuleFinding> {
    let root = case_root(name);
    let path = root.join(".github/workflows/ci.yml");
    let (uses_re, sha_re, version_re) = build_patterns();
    let exclude_set = build_exclude_globset(&[]);
    check_file(&path, &root, &exclude_set, &uses_re, &sha_re, &version_re)
}

#[test]
fn pass_fixture_produces_no_findings() {
    let root = fixture("pass");
    let config = config_with_options("{}");
    let files = vec![root.join(".github").join("workflows").join("ci.yml")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn fail_fixture_produces_findings() {
    let root = fixture("fail");
    let config = config_with_options("{}");
    let files = vec![root.join(".github").join("workflows").join("ci.yml")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(!findings.is_empty(), "expected findings");
    assert!(
        findings.iter().all(|f| f.rule == RULE_ID),
        "all findings should have correct rule id"
    );
}

#[test]
fn local_action_pass_fixture_produces_no_findings() {
    let root = fixture("local-action-pass");
    let config = config_with_options("{}");
    let files = vec![root.join(".github").join("workflows").join("ci.yml")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn correctly_pinned_passes() {
    let findings = run_on_workflow("pinned-version-comment");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn double_quoted_uses_passes() {
    let findings = run_on_workflow("double-quoted-uses");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn single_quoted_uses_passes() {
    let findings = run_on_workflow("single-quoted-uses");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn double_quoted_tag_ref_flagged() {
    let findings = run_on_workflow("double-quoted-tag");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("not a 40-char commit SHA"));
}

#[test]
fn build_metadata_version_comment_passes() {
    let findings = run_on_workflow("build-metadata-comment");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn uppercase_sha_passes() {
    let findings = run_on_workflow("uppercase-sha");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn tag_ref_flagged() {
    let findings = run_on_workflow("tag-ref");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 1);
    assert!(findings[0].message.contains("not a 40-char commit SHA"));
}

#[test]
fn branch_ref_flagged() {
    let findings = run_on_workflow("branch-ref");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("not a 40-char commit SHA"));
}

#[test]
fn sha_without_comment_flagged() {
    let findings = run_on_workflow("sha-without-comment");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("trailing comment must be"));
}

#[test]
fn sha_with_bad_comment_flagged() {
    let findings = run_on_workflow("sha-bad-comment");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("trailing comment must be"));
}

#[test]
fn sha_with_version_comment_passes() {
    let findings = run_on_workflow("pinned-version-comment");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn major_only_version_comment_passes() {
    let findings = run_on_workflow("major-only-comment");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn numeric_version_comment_passes() {
    // Rust versions like 1.87.0 (no v prefix) must also pass
    let findings = run_on_workflow("numeric-version-comment");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn local_action_exempt() {
    let findings = run_on_workflow("local-action");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn docker_ref_exempt() {
    let findings = run_on_workflow("docker-ref");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn non_workflow_yaml_not_checked() {
    let root = case_root("non-workflow-yaml");
    let path = root.join("config.yml");
    let (uses_re, sha_re, version_re) = build_patterns();
    let exclude_set = build_exclude_globset(&[]);
    let findings = check_file(&path, &root, &exclude_set, &uses_re, &sha_re, &version_re);
    assert!(
        findings.is_empty(),
        "non-workflow yaml should not be checked"
    );
}

#[test]
fn short_sha_flagged() {
    // 39-char SHA (one short) must fail
    let findings = run_on_workflow("short-sha");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("not a 40-char commit SHA"));
}

#[test]
fn exclude_path_option_skips_file() {
    let root = fixture("fail");
    let config = config_with_options("excludePaths: ['.github/workflows/ci.yml']");
    let files = vec![root.join(".github").join("workflows").join("ci.yml")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "excluded path should produce no findings"
    );
}

#[test]
fn multiple_violations_reported() {
    // Keep valid YAML here; malformed-workflow uncertainty has its own saved regression.
    let root = crate::test_support::rule_fixture_root("file-input-analysis");
    let path = root.join(".github/workflows/multiple-unpinned.yml");
    let (uses_re, sha_re, version_re) = build_patterns();
    let findings = check_file(
        &path,
        &root,
        &build_exclude_globset(&[]),
        &uses_re,
        &sha_re,
        &version_re,
    );
    assert_eq!(findings.len(), 2, "expected 2 violations: {findings:?}");
}

#[test]
fn reusable_workflow_uses_field_checked() {
    // `uses:` at job level (reusable workflows) should also be checked
    let findings = run_on_workflow("reusable-workflow");
    assert_eq!(findings.len(), 1);
}

#[test]
fn check_entry_point_uses_discovery() {
    // smoke-test the public `check()` entry point; uses the pass fixture
    // which enables the rule and has only pinned actions
    let root = fixture("pass");
    let config = config_with_options("{}");
    let findings = super::check(&root, &config).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn non_yaml_extension_not_checked() {
    // A file with a non-yml/yaml extension in .github/workflows/ must be skipped
    let root = case_root("non-yaml-extension");
    let path = root.join(".github/workflows/ci.sh");
    let (uses_re, sha_re, version_re) = build_patterns();
    let exclude_set = build_exclude_globset(&[]);
    let findings = check_file(&path, &root, &exclude_set, &uses_re, &sha_re, &version_re);
    assert!(
        findings.is_empty(),
        "non-yaml extension should not be checked"
    );
}

#[test]
fn composite_action_yaml_extension_checked() {
    // .github/actions/**/action.yaml (note: .yaml not .yml) must be checked
    let root = case_root("composite-action-yaml");
    let path = root.join(".github/actions/my-action/action.yaml");
    let (uses_re, sha_re, version_re) = build_patterns();
    let exclude_set = build_exclude_globset(&[]);
    let findings = check_file(&path, &root, &exclude_set, &uses_re, &sha_re, &version_re);
    assert_eq!(findings.len(), 1, "composite action.yaml must be checked");
}

#[test]
fn unreadable_file_reports_unanalyzable_input() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(".github/workflows/missing.yml");
    let (uses_re, sha_re, version_re) = build_patterns();
    let exclude_set = build_exclude_globset(&[]);
    let findings = check_file(
        &path,
        tmp.path(),
        &exclude_set,
        &uses_re,
        &sha_re,
        &version_re,
    );
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable"));
}

#[test]
fn block_header_accepts_one_chomp_or_indent() {
    assert!(yaml_uses::is_block_header("|+2"));
    assert!(yaml_uses::is_block_header("|2+"));
    assert!(yaml_uses::is_block_header("|-"));
    assert!(!yaml_uses::is_block_header("|not"));
    assert!(!yaml_uses::is_block_header("|12"));
    assert!(!yaml_uses::is_block_header("|+-"));
    assert!(!yaml_uses::is_block_header("plain"));
}

#[test]
fn non_string_job_id_still_checks_the_step_line() {
    // YAML reads `1` as a number, so the job is not walked by name. The step
    // line is still a `uses` hit.
    let findings = check_yaml_shape("numeric-job-fail.yml");
    assert_eq!(lines_of(&findings), vec![5], "{findings:?}");
    assert!(findings[0].message.contains("actions/checkout@v4"));
}

#[test]
fn aliased_steps_and_job_named_env_are_checked() {
    let aliased = check_yaml_shape("aliased-steps-fail.yml");
    assert_eq!(lines_of(&aliased), vec![10], "{aliased:?}");
    assert!(aliased[0].message.contains("actions/checkout@v4"));
    let named = check_yaml_shape("job-named-env-fail.yml");
    assert_eq!(lines_of(&named), vec![5, 8], "{named:?}");
    assert!(named[0].message.contains("actions/checkout@v4"));
    assert!(named[1].message.contains("actions/setup-node@v4"));
    // A job id of steps or jobs is still a job, so its workflow uses is checked.
    let structural = check_yaml_shape("job-named-steps-fail.yml");
    assert_eq!(lines_of(&structural), vec![4, 6], "{structural:?}");
}

#[test]
fn uses_line_not_matching_regex_skipped() {
    // A line containing "uses:" but not as a YAML step key must not produce findings.
    // The regex requires `^\s*-?\s*uses:\s+<value>`, so bare `uses:` with nothing after is skipped.
    let findings = run_on_workflow("comment-uses");
    assert!(findings.is_empty(), "{findings:?}");
}

fn check_yaml_shape(name: &str) -> Vec<RuleFinding> {
    let root = fixture("yaml-shapes");
    let config = config_with_options("{}");
    let files = vec![root.join(".github").join("workflows").join(name)];
    check_with_files(&root, &config, &files).unwrap()
}

fn lines_of(findings: &[RuleFinding]) -> Vec<usize> {
    findings.iter().map(|finding| finding.line).collect()
}

mod yaml_shapes;
