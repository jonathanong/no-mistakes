use super::*;
use std::path::PathBuf;

fn written_ci(source: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(".github/workflows/ci.yml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, source).unwrap();
    (tmp, path)
}

fn scan_written(source: &str, defer_suppression: bool) -> Vec<RuleFinding> {
    let (tmp, path) = written_ci(source);
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let opts = compile_options(serde_yaml::from_str(OPTIONS).unwrap());
    scan::check_file(tmp.path(), &path, &opts, &sources, defer_suppression)
}

fn check_written(source: &str, defer_suppression: bool) -> Vec<RuleFinding> {
    let (tmp, path) = written_ci(source);
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    check_with_files_and_sources(
        tmp.path(),
        &config(OPTIONS),
        std::slice::from_ref(&path),
        &sources,
        defer_suppression,
    )
    .unwrap()
}

const DISABLED_THIRD_PARTY: &str = "\
# no-mistakes-disable-file github-actions-action-timeout-pair
jobs:
  deploy:
    steps:
      - uses: aws-actions/configure-aws-credentials@v4
        timeout-minutes: 2
";

#[test]
fn missing_nested_timeout_is_on_the_violating_step() {
    let source = r#"
jobs:
  deploy:
    steps:
      - uses: aws-actions/configure-aws-credentials@v4
        timeout-minutes: 2
        with:
          action-timeout-s: 90
      - uses: aws-actions/configure-aws-credentials@v4
        timeout-minutes: 2
"#;
    let findings = parsed(".github/workflows/ci.yml", source, OPTIONS);
    assert_eq!(findings.len(), 1, "{findings:?}");
    let expected = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("configure-aws-credentials"))
        .nth(1)
        .map(|(index, _)| index + 1)
        .unwrap();
    assert_eq!(findings[0].line, expected);
}

#[test]
fn nested_timeout_line_falls_back_without_steps_or_keys() {
    assert_eq!(
        super::super::line::nested_timeout_line("name: ci\n", "missing", 0, "action-timeout-s"),
        1
    );
    let with_only = "runs:\n  using: composite\n  steps:\n    - with:\n        role: x\n";
    assert_eq!(
        super::super::line::nested_timeout_line(with_only, "(composite)", 0, "action-timeout-s"),
        4
    );
    let name_only =
        "jobs:\n  deploy:\n    steps:\n      - name: only\n        timeout-minutes: 2\n";
    assert_eq!(
        super::super::line::nested_timeout_line(name_only, "deploy", 0, "action-timeout-s"),
        4
    );
    let gapped = "jobs:\n  deploy:\n    steps:\n      - uses: x\n\n      - uses: y\n";
    assert_eq!(
        super::super::line::nested_timeout_line(gapped, "deploy", 1, "action-timeout-s"),
        6
    );
    assert_eq!(
        super::super::line::nested_timeout_line(gapped, "deploy", 8, "action-timeout-s"),
        1
    );
    assert_eq!(
        super::super::line::step_key_line("name: ci\n", "missing", 0, "timeout-minutes"),
        1
    );
}

#[test]
fn nested_timeout_line_skips_nested_lists_and_later_jobs() {
    let source = r#"
jobs:
  deploy:
    steps:
      - uses: aws-actions/configure-aws-credentials@v4
        timeout-minutes: 2
        with:
          action-timeout-s: 90
          items:
            - one
      - uses: aws-actions/configure-aws-credentials@v4
        timeout-minutes: 2
  other:
    name: x
"#;
    let findings = parsed(".github/workflows/ci.yml", source, OPTIONS);
    assert_eq!(findings.len(), 1, "{findings:?}");
    let expected = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("configure-aws-credentials"))
        .nth(1)
        .map(|(index, _)| index + 1)
        .unwrap();
    assert_eq!(findings[0].line, expected);
}

#[test]
fn nested_composite_under_wrapper_dir_is_forbidden() {
    let source = r#"
runs:
  using: composite
  steps:
    - uses: ./.github/actions/setup-aws
"#;
    let findings = parsed(
        ".github/actions/setup-aws/nested/action.yml",
        source,
        OPTIONS,
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("nests a configured action"));
}

#[test]
fn wrapper_action_yaml_still_requires_nested_input() {
    let source = r#"
runs:
  using: composite
  steps:
    - name: inner
      uses: aws-actions/configure-aws-credentials@v4
"#;
    let findings = parsed(".github/actions/setup-aws/action.yaml", source, OPTIONS);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("action-timeout-s"));
    assert_eq!(findings[0].line, 6);
}

#[test]
fn disable_file_comment_is_kept_when_deferred() {
    let suppressed = scan_written(DISABLED_THIRD_PARTY, false);
    assert!(suppressed.is_empty(), "{suppressed:?}");
    let deferred = scan_written(DISABLED_THIRD_PARTY, true);
    assert_eq!(deferred.len(), 1, "{deferred:?}");
    assert!(check_written(DISABLED_THIRD_PARTY, false).is_empty());
    assert_eq!(check_written(DISABLED_THIRD_PARTY, true).len(), 1);
}

#[test]
fn forbid_nested_false_still_checks_third_party_nested_input() {
    let source = r#"
runs:
  using: composite
  steps:
    - uses: aws-actions/configure-aws-credentials@v4
"#;
    let yaml = r#"
uses:
  - ./.github/actions/setup-aws
  - aws-actions/configure-aws-credentials@
nestedInput: action-timeout-s
nestedTimeoutSeconds: 90
forbidNestedInComposite: false
"#;
    let findings = parsed(".github/actions/other/action.yml", source, yaml);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("action-timeout-s"));
}

#[test]
fn timeout_minutes_line_is_on_the_violating_step() {
    let source = r#"
jobs:
  deploy:
    steps:
      - uses: aws-actions/configure-aws-credentials@v4
        timeout-minutes: 2
        with:
          action-timeout-s: 90
      - uses: aws-actions/configure-aws-credentials@v4
        with:
          action-timeout-s: 90
"#;
    let findings = parsed(".github/workflows/ci.yml", source, OPTIONS);
    assert_eq!(findings.len(), 1, "{findings:?}");
    let expected = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("configure-aws-credentials"))
        .nth(1)
        .map(|(index, _)| index + 1)
        .unwrap();
    assert_eq!(findings[0].line, expected);
}

#[test]
fn wrapper_identity_uses_project_target_root() {
    // Include globs are project-relative; wrapper identity must use that same
    // target root or packages/app/.github/actions/setup-aws is not a wrapper.
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp
        .path()
        .join("packages/app/.github/actions/setup-aws/action.yml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        "runs:\n  using: composite\n  steps:\n    - uses: aws-actions/configure-aws-credentials@v4\n",
    )
    .unwrap();
    let config = NoMistakesConfig {
        projects: [(
            "app".to_string(),
            crate::config::v2::schema::Project {
                root: Some("packages/app".to_string()),
                ..Default::default()
            },
        )]
        .into(),
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            projects: vec!["app".to_string()],
            options: serde_yaml::from_str(OPTIONS).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let findings = check_with_files(tmp.path(), &config, std::slice::from_ref(&path)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("action-timeout-s"),
        "{findings:?}"
    );
    assert!(!findings[0].message.contains("nests a configured action"));
}

#[test]
fn disable_next_line_on_uses_is_kept_when_deferred() {
    let source = r#"
jobs:
  deploy:
    steps:
      # no-mistakes-disable-next-line github-actions-action-timeout-pair
      - uses: aws-actions/configure-aws-credentials@v4
        timeout-minutes: 2
"#;
    let suppressed = scan_written(source, false);
    assert!(suppressed.is_empty(), "{suppressed:?}");
    let deferred = scan_written(source, true);
    assert_eq!(deferred.len(), 1, "{deferred:?}");
    let expected = source
        .lines()
        .position(|line| line.contains("uses:"))
        .map(|index| index + 1)
        .unwrap();
    assert_eq!(deferred[0].line, expected);
}
