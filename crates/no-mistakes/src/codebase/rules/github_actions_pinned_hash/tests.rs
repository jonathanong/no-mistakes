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

fn run_on_workflow(content: &str) -> Vec<RuleFinding> {
    let tmp = tempfile::tempdir().unwrap();
    let wf_dir = tmp.path().join(".github/workflows");
    std::fs::create_dir_all(&wf_dir).unwrap();
    let path = wf_dir.join("ci.yml");
    std::fs::write(&path, content).unwrap();
    let (uses_re, sha_re, version_re) = build_patterns();
    let exclude_set = build_exclude_globset(&[]);
    check_file(
        &path,
        tmp.path(),
        &exclude_set,
        &uses_re,
        &sha_re,
        &version_re,
    )
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
    let findings = run_on_workflow(
        "      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2\n",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn double_quoted_uses_passes() {
    let findings = run_on_workflow(
        "      - uses: \"actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd\" # v6.0.2\n",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn single_quoted_uses_passes() {
    let findings = run_on_workflow(
        "      - uses: 'actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd' # v6.0.2\n",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn double_quoted_tag_ref_flagged() {
    let findings = run_on_workflow("      - uses: \"actions/checkout@v6.0.2\" # v6.0.2\n");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("not a 40-char commit SHA"));
}

#[test]
fn build_metadata_version_comment_passes() {
    let findings = run_on_workflow(
        "      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v1.0.0-beta+build.1\n",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn uppercase_sha_passes() {
    let findings = run_on_workflow(
        "      - uses: actions/checkout@DE0FAC2E4500DABE0009E67214FF5F5447CE83DD # v6.0.2\n",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn tag_ref_flagged() {
    let findings = run_on_workflow("      - uses: actions/checkout@v6.0.2\n");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 1);
    assert!(findings[0].message.contains("not a 40-char commit SHA"));
}

#[test]
fn branch_ref_flagged() {
    let findings = run_on_workflow("      - uses: dtolnay/rust-toolchain@stable\n");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("not a 40-char commit SHA"));
}

#[test]
fn sha_without_comment_flagged() {
    let findings = run_on_workflow(
        "      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd\n",
    );
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("trailing comment must be"));
}

#[test]
fn sha_with_bad_comment_flagged() {
    let findings = run_on_workflow(
        "      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # main\n",
    );
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("trailing comment must be"));
}

#[test]
fn sha_with_version_comment_passes() {
    let findings = run_on_workflow(
        "      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2\n",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn major_only_version_comment_passes() {
    let findings = run_on_workflow(
        "      - uses: Swatinem/rust-cache@e18b497796c12c097a38f9edb9d0641fb99eee32 # v2\n",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn numeric_version_comment_passes() {
    // Rust versions like 1.87.0 (no v prefix) must also pass
    let findings = run_on_workflow(
        "      - uses: dtolnay/rust-toolchain@6190aa5fb88a88ee71c12769924bbe63a9ab152e # 1.96.0\n",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn local_action_exempt() {
    let findings = run_on_workflow("      - uses: ./.github/actions/my-action\n");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn docker_ref_exempt() {
    let findings = run_on_workflow("      - uses: docker://alpine:3.20\n");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn non_workflow_yaml_not_checked() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("config.yml");
    std::fs::write(&path, "      - uses: actions/checkout@v6.0.2\n").unwrap();
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
    assert!(
        findings.is_empty(),
        "non-workflow yaml should not be checked"
    );
}

#[test]
fn short_sha_flagged() {
    // 39-char SHA (one short) must fail
    let findings =
        run_on_workflow("      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83d\n");
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
    let content = "\
      - uses: actions/checkout@v6.0.2
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
";
    let findings = run_on_workflow(content);
    assert_eq!(findings.len(), 2, "expected 2 violations: {findings:?}");
}

#[test]
fn reusable_workflow_uses_field_checked() {
    // `uses:` at job level (reusable workflows) should also be checked
    let findings = run_on_workflow("    uses: my-org/my-repo/.github/workflows/build.yml@main\n");
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
    let tmp = tempfile::tempdir().unwrap();
    let wf_dir = tmp.path().join(".github/workflows");
    std::fs::create_dir_all(&wf_dir).unwrap();
    let path = wf_dir.join("ci.sh");
    std::fs::write(&path, "uses: actions/checkout@v6.0.2\n").unwrap();
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
    assert!(
        findings.is_empty(),
        "non-yaml extension should not be checked"
    );
}

#[test]
fn composite_action_yaml_extension_checked() {
    // .github/actions/**/action.yaml (note: .yaml not .yml) must be checked
    let tmp = tempfile::tempdir().unwrap();
    let action_dir = tmp.path().join(".github/actions/my-action");
    std::fs::create_dir_all(&action_dir).unwrap();
    let path = action_dir.join("action.yaml");
    std::fs::write(&path, "      - uses: actions/checkout@v6.0.2\n").unwrap();
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
    assert_eq!(findings.len(), 1, "composite action.yaml must be checked");
}

#[test]
fn unreadable_file_produces_no_findings() {
    // check_file must return empty when the path does not exist
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
    assert!(
        findings.is_empty(),
        "missing file should produce no findings"
    );
}

#[test]
fn uses_line_not_matching_regex_skipped() {
    // A line containing "uses:" but not as a YAML step key must not produce findings.
    // The regex requires `^\s*-?\s*uses:\s+<value>`, so bare `uses:` with nothing after is skipped.
    let findings = run_on_workflow("      # uses: some comment reference\n");
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

#[test]
fn yaml_flow_quoted_and_next_line_uses_are_checked() {
    // The tag on the last line of flow-pass proves the document parsed, so the
    // pinned, local, parent, and Docker flow entries were judged, not skipped.
    let flow = check_yaml_shape("flow-fail.yml");
    assert_eq!(lines_of(&flow), vec![1], "{flow:?}");
    assert!(flow[0].message.contains("actions/checkout@v4"));
    assert!(flow[0].message.contains("not a 40-char commit SHA"));

    let next_line = check_yaml_shape("next-line-fail.yml");
    assert_eq!(lines_of(&next_line), vec![2], "{next_line:?}");
    assert!(next_line[0].message.contains("actions/checkout@v4"));

    let quoted = check_yaml_shape("quoted-key-fail.yml");
    assert_eq!(lines_of(&quoted), vec![1], "{quoted:?}");
    assert!(quoted[0].message.contains("not a 40-char commit SHA"));

    let passed = check_yaml_shape("flow-pass.yml");
    assert_eq!(lines_of(&passed), vec![8], "{passed:?}");
    assert!(passed[0].message.contains("actions/checkout@v4"));
    assert!(!passed[0].message.contains("de0fac2e"));

    let next_pass = check_yaml_shape("next-line-pass.yml");
    assert_eq!(lines_of(&next_pass), vec![3], "{next_pass:?}");

    let multi = check_yaml_shape("multi-doc-fail.yml");
    assert_eq!(lines_of(&multi), vec![1, 3], "{multi:?}");

    let dedup = check_yaml_shape("dedup-fail.yml");
    assert_eq!(lines_of(&dedup), vec![1, 2], "{dedup:?}");
}
