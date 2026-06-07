use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

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
    check_file(&path, tmp.path(), &exclude_set, &uses_re, &sha_re, &version_re)
}

#[test]
fn pass_fixture_produces_no_findings() {
    let root = fixture("pass");
    let config = config_with_options("{}");
    let files = vec![root
        .join(".github")
        .join("workflows")
        .join("ci.yml")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn fail_fixture_produces_findings() {
    let root = fixture("fail");
    let config = config_with_options("{}");
    let files = vec![root
        .join(".github")
        .join("workflows")
        .join("ci.yml")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(!findings.is_empty(), "expected findings");
    assert!(
        findings
            .iter()
            .all(|f| f.rule == RULE_ID),
        "all findings should have correct rule id"
    );
}

#[test]
fn local_action_pass_fixture_produces_no_findings() {
    let root = fixture("local-action-pass");
    let config = config_with_options("{}");
    let files = vec![root
        .join(".github")
        .join("workflows")
        .join("ci.yml")];
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
    std::fs::write(
        &path,
        "      - uses: actions/checkout@v6.0.2\n",
    )
    .unwrap();
    let (uses_re, sha_re, version_re) = build_patterns();
    let exclude_set = build_exclude_globset(&[]);
    let findings = check_file(&path, tmp.path(), &exclude_set, &uses_re, &sha_re, &version_re);
    assert!(findings.is_empty(), "non-workflow yaml should not be checked");
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
    let files = vec![root
        .join(".github")
        .join("workflows")
        .join("ci.yml")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "excluded path should produce no findings");
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
