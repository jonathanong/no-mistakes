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

    // The pinned block uses is already judged. Repeating its ref in run, name,
    // or a comment must not move the finding. The later flow uses still fails.
    let echoed = check_yaml_shape("echo-ref-fail.yml");
    assert_eq!(lines_of(&echoed), vec![5], "{echoed:?}");
    assert!(echoed[0].message.contains("actions/checkout@v4"));
    assert!(!echoed[0].message.contains("de0fac2e"));

    // `from_str` rejects the whole file. Document 1's flow uses must still fail.
    let multi_flow = check_yaml_shape("multi-doc-flow-fail.yml");
    assert_eq!(lines_of(&multi_flow), vec![1], "{multi_flow:?}");
    assert!(multi_flow[0].message.contains("actions/checkout@v4"));

    let folded = check_yaml_shape("folded-flow-fail.yml");
    assert_eq!(lines_of(&folded), vec![1], "{folded:?}");
    assert!(folded[0].message.contains("actions/checkout@v4"));

    let anchored = check_yaml_shape("anchor-flow-fail.yml");
    assert_eq!(lines_of(&anchored), vec![1], "{anchored:?}");
    assert!(anchored[0].message.contains("actions/checkout@v4"));
    assert!(!anchored[0].message.contains("&checkout"));

    let input = check_yaml_shape("with-uses-pass.yml");
    assert_eq!(lines_of(&input), Vec::<usize>::new(), "{input:?}");

    // The input comment shares the resolved ref. The later action still needs
    // its own version comment.
    let stolen = check_yaml_shape("with-comment-fail.yml");
    assert_eq!(lines_of(&stolen), vec![5], "{stolen:?}");
    assert!(stolen[0]
        .message
        .contains("de0fac2e4500dabe0009e67214ff5f5447ce83dd"));
    assert!(stolen[0].message.contains("trailing comment"));

    let alias = check_yaml_shape("alias-fail.yml");
    assert_eq!(lines_of(&alias), vec![3], "{alias:?}");
    assert!(alias[0]
        .message
        .contains("de0fac2e4500dabe0009e67214ff5f5447ce83dd"));
    assert!(alias[0].message.contains("trailing comment"));
    assert!(!alias[0].message.contains("*checkout"));

    let block_input = check_yaml_shape("with-block-uses-pass.yml");
    assert_eq!(
        lines_of(&block_input),
        Vec::<usize>::new(),
        "{block_input:?}"
    );

    let nested = check_yaml_shape("nested-uses-pass.yml");
    assert_eq!(lines_of(&nested), Vec::<usize>::new(), "{nested:?}");

    // `{ uses:` with the ref on the next line still fails.
    let broken = check_yaml_shape("flow-break-fail.yml");
    assert_eq!(lines_of(&broken), vec![3], "{broken:?}");
    assert!(broken[0].message.contains("actions/checkout@v4"));

    // A flow `with` broken across lines is an input. Only the unpinned action fails.
    let flow_input = check_yaml_shape("flow-with-break-fail.yml");
    assert_eq!(lines_of(&flow_input), vec![11], "{flow_input:?}");
    assert!(flow_input[0].message.contains("actions/checkout@v4"));
    assert!(!flow_input[0].message.contains("de0fac2e"));

    // Two uses values on one line both stay paired. The tag is the finding.
    let same_line = check_yaml_shape("same-line-uses-fail.yml");
    assert_eq!(lines_of(&same_line), vec![2], "{same_line:?}");
    assert!(same_line[0].message.contains("actions/checkout@v4"));
    assert!(!same_line[0].message.contains("de0fac2e"));

    // A run script is not an action line. The block uses value still fails.
    let script = check_yaml_shape("block-script-fail.yml");
    assert_eq!(lines_of(&script), vec![8, 11], "{script:?}");
    assert!(script
        .iter()
        .all(|finding| finding.message.contains("actions/checkout@v4")));
    assert!(script
        .iter()
        .all(|finding| !finding.message.contains("de0fac2e")));

    // The matrix comment must not validate the step, and include is not a finding.
    let matrix = check_yaml_shape("matrix-uses-fail.yml");
    assert_eq!(lines_of(&matrix), vec![9, 17], "{matrix:?}");
    assert!(matrix[0]
        .message
        .contains("de0fac2e4500dabe0009e67214ff5f5447ce83dd"));
    assert!(matrix[0].message.contains("trailing comment"));
    assert!(!matrix[0].message.contains("ubuntu-latest"));
    assert!(matrix[1].message.contains("actions/checkout@v4"));
    assert!(!matrix[1].message.contains("ubuntu-latest"));

    // `\"` inside the name is not the end of the scalar.
    let escaped = check_yaml_shape("quote-escape-fail.yml");
    assert_eq!(lines_of(&escaped), vec![2], "{escaped:?}");
    assert!(escaped[0].message.contains("actions/checkout@v4"));

    // A comment on `|` must not move the step finding onto the script line.
    let header = check_yaml_shape("block-header-comment-fail.yml");
    assert_eq!(lines_of(&header), vec![4, 7], "{header:?}");
    assert!(header
        .iter()
        .all(|finding| finding.message.contains("actions/checkout@v4")));
    assert!(header
        .iter()
        .all(|finding| !finding.message.contains("de0fac2e")));

    // `uses :` with a space before the colon is still an action key.
    let spaced = check_yaml_shape("space-before-colon-fail.yml");
    assert_eq!(lines_of(&spaced), vec![2, 3], "{spaced:?}");
    assert!(spaced[0].message.contains("actions/checkout@v4"));
    assert!(spaced[1].message.contains("actions/checkout@v3"));

    // `#` after an escaped quote stays inside the name. The pinned line passes.
    let comment = check_yaml_shape("escaped-comment-fail.yml");
    assert_eq!(lines_of(&comment), vec![3], "{comment:?}");
    assert!(comment[0].message.contains("actions/checkout@v4"));
    assert!(!comment[0].message.contains("trailing comment"));
    assert!(!comment[0].message.contains("de0fac2e"));

    // The alias line is the step. The anchor's tag must not be dropped.
    let alias_step = check_yaml_shape("mapping-alias-fail.yml");
    assert_eq!(lines_of(&alias_step), vec![8, 16], "{alias_step:?}");
    assert!(alias_step[0].message.contains("actions/checkout@v4"));
    assert!(alias_step[1].message.contains("actions/checkout@v3"));

    // `\u0065` in the quoted key is `e`, so the unpinned action is still found.
    let encoded = check_yaml_shape("quoted-escape-key-fail.yml");
    assert_eq!(lines_of(&encoded), vec![2], "{encoded:?}");
    assert!(encoded[0].message.contains("actions/checkout@v4"));

    // `foo#bar` is one plain scalar. The unpinned action after it still fails.
    let hashed = check_yaml_shape("plain-hash-fail.yml");
    assert_eq!(lines_of(&hashed), vec![2], "{hashed:?}");
    assert!(hashed[0].message.contains("actions/checkout@v4"));

    // The header comment covers the pinned block value. The bare header does not.
    let chomp = check_yaml_shape("block-chomp-comment-fail.yml");
    assert_eq!(lines_of(&chomp), vec![6, 9], "{chomp:?}");
    assert!(chomp[0].message.contains("trailing comment"));
    assert!(chomp[0].message.contains("de0fac2e"));
    assert!(chomp[1].message.contains("actions/checkout@v4"));
    assert!(!chomp[1].message.contains("trailing comment"));
}

#[test]
fn same_column_and_bare_dash_steps_are_checked() {
    // A dash in the steps column is still a step. The include field is not.
    let same = check_yaml_shape("same-indent-steps-fail.yml");
    assert_eq!(lines_of(&same), vec![10, 11], "{same:?}");
    assert!(same[0].message.contains("actions/checkout@v4"));
    assert!(same[1].message.contains("actions/setup-node@v4"));

    // The key on the line after a bare dash, including a commented dash, is the step.
    let bare = check_yaml_shape("bare-dash-step-fail.yml");
    assert_eq!(lines_of(&bare), vec![6, 8], "{bare:?}");
    assert!(bare[0].message.contains("actions/checkout@v4"));
    assert!(bare[1].message.contains("actions/setup-node@v4"));
}

#[test]
fn anchor_only_and_plain_apostrophe_steps_are_checked() {
    let anchored = check_yaml_shape("anchor-step-fail.yml");
    assert_eq!(lines_of(&anchored), vec![6, 7], "{anchored:?}");
    assert!(anchored[0].message.contains("actions/checkout@v4"));
    assert!(anchored[1].message.contains("actions/setup-node@v4"));

    // The apostrophe stays inside the plain name, so the next flow step fails.
    let named = check_yaml_shape("plain-apostrophe-fail.yml");
    assert_eq!(lines_of(&named), vec![3], "{named:?}");
    assert!(named[0].message.contains("actions/checkout@v4"));

    // The apostrophe stays in the name, so the trailing version comment counts.
    let comment = check_yaml_shape("plain-apostrophe-comment-fail.yml");
    assert_eq!(lines_of(&comment), vec![4], "{comment:?}");
    assert!(comment[0].message.contains("trailing comment"));
    assert!(comment[0].message.contains("de0fac2e"));
}
