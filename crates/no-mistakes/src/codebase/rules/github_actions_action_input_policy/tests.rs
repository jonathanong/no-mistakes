use super::policy;
use super::{check_with_files, scan, Options, RuleFinding, RULE_ID};
use crate::config::v2::NoMistakesConfig;
use serde_yaml::Value;
use std::path::PathBuf;

#[path = "tests/allowlist.rs"]
mod allowlist;
#[path = "tests/calls.rs"]
mod calls;

fn rule_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-action-input-policy")
}

fn load_config(name: &str) -> NoMistakesConfig {
    let path = rule_root().join("configs").join(name);
    serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn workflow(case: &str) -> (PathBuf, PathBuf) {
    let root = rule_root().join("fixture").join(case);
    let name = if case == "invalid" {
        "broken.yml"
    } else {
        "widgets.yml"
    };
    (root.clone(), root.join(".github/workflows").join(name))
}

fn run_case(case: &str, config: &str) -> Vec<RuleFinding> {
    let (root, file) = workflow(case);
    check_with_files(&root, &load_config(config), &[file]).unwrap()
}

fn parsed(source: &str, options: &str) -> Vec<RuleFinding> {
    let opts = policy::compile(serde_yaml::from_str(options).unwrap()).unwrap();
    let value: Value = serde_yaml::from_str(source).unwrap();
    scan::check_parsed("widgets.yml", source, &value, &opts)
}

const POLICY: &str = r#"
actions:
  - uses: example/publish-widget@v1
    require:
      keep-days: 1
      replace: true
"#;

#[test]
fn violations_report_required_banned_and_pinned_refs_only() {
    let findings = run_case("violations", "policy.yml");
    assert_eq!(findings.len(), 8, "{findings:?}");
    let messages: Vec<_> = findings
        .iter()
        .map(|finding| finding.message.as_str())
        .collect();
    assert!(messages
        .iter()
        .any(|message| message.contains("keep-days") && message.contains("30")));
    assert!(messages
        .iter()
        .any(|message| message.contains("replace") && message.contains("false")));
    assert!(messages
        .iter()
        .any(|message| message.contains("banned input `cache`")));
    assert!(messages
        .iter()
        .any(|message| message.contains("banned input `runtime-version`")));
    assert!(messages
        .iter()
        .any(|message| message.contains("runtime-version-file")));
    assert!(messages
        .iter()
        .any(|message| message.contains("package-manager-cache")));
    assert!(messages
        .iter()
        .any(|message| message.contains("banned input `overwrite`")));
    assert!(messages
        .iter()
        .all(|message| !message.contains("checkout-orders")));
    assert!(messages
        .iter()
        .all(|message| !message.contains("matrix.widget")));
    assert!(findings.iter().all(|finding| finding.rule == RULE_ID));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("publish pinned widgets")));
}

#[test]
fn matching_values_and_unnamed_actions_stay_silent() {
    assert!(run_case("pass", "policy.yml").is_empty());
}

#[test]
fn empty_actions_and_include_miss_stay_silent() {
    assert!(run_case("violations", "empty-actions.yml").is_empty());
    assert!(run_case("violations", "include-miss.yml").is_empty());
}

#[test]
fn invalid_yaml_and_missing_file_stay_silent() {
    assert!(run_case("invalid", "policy.yml").is_empty());
    let (root, file) = workflow("violations");
    let missing = file.with_file_name("absent.yml");
    let opts = policy::compile(serde_yaml::from_str::<Options>(POLICY).unwrap()).unwrap();
    let sources = super::super::source_store_for_files(std::slice::from_ref(&missing));
    assert!(scan::check_file(&root, &missing, &opts, &sources, false).is_empty());
}

#[test]
fn line_disable_hides_one_step_and_file_disable_hides_the_file() {
    let findings = run_case("disabled-line", "policy.yml");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("open widget"), "{findings:?}");
    assert!(run_case("disabled-file", "policy.yml").is_empty());
}

#[test]
fn deferred_suppression_keeps_a_disabled_file_finding() {
    let (root, file) = workflow("disabled-file");
    let opts = policy::compile(serde_yaml::from_str::<Options>(POLICY).unwrap()).unwrap();
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    let findings = scan::check_file(&root, &file, &opts, &sources, true);
    assert_eq!(findings.len(), 2, "{findings:?}");
}

#[test]
fn families_ignore_refs_and_reject_expressions() {
    assert_eq!(
        policy::family_of("example/publish-widget@v1/"),
        Some("example/publish-widget".to_string())
    );
    assert_eq!(
        policy::family_of("docker://orders:1"),
        Some("docker://orders:1".to_string())
    );
    assert_eq!(policy::family_of(" ${{ matrix.widget }} "), None);
    assert_eq!(policy::family_of(" @v1"), None);
}

#[test]
fn config_rejects_contradictions_and_non_scalars() {
    let both = r#"
actions:
  - uses: example/publish-widget
    ban: [keep-days]
    require:
      keep-days: 1
"#;
    let conflict = r#"
actions:
  - uses: example/publish-widget
    require:
      keep-days: 1
  - uses: example/publish-widget@v1
    require:
      keep-days: 2
"#;
    let nested = r#"
actions:
  - uses: example/publish-widget
    require:
      keep-days:
        days: 1
"#;
    let dynamic = "actions:\n  - uses: ${{ matrix.widget }}\n";
    let blank = "actions:\n  - uses: \"   \"\n    require:\n      keep-days: 1\n";
    let empty_key = "actions:\n  - uses: example/publish-widget\n    require:\n      \"\": 1\n";
    let ban_after_require = r#"
actions:
  - uses: example/publish-widget
    require:
      keep-days: 1
  - uses: example/publish-widget@v1
    ban: [keep-days]
"#;
    for yaml in [
        both,
        conflict,
        nested,
        dynamic,
        blank,
        empty_key,
        ban_after_require,
    ] {
        let error = policy::compile(serde_yaml::from_str(yaml).unwrap())
            .unwrap_err()
            .to_string();
        assert!(error.contains(RULE_ID), "{error}");
    }

    let multiple = r#"
actions:
  - uses: "${{ matrix.widget }}"
  - uses: example/publish-widget
    allowedInputs: [name]
    ban: [name]
    require:
      format: json
"#;
    let error = policy::compile(serde_yaml::from_str(multiple).unwrap())
        .unwrap_err()
        .to_string();
    let lines: Vec<_> = error.lines().collect();
    assert_eq!(lines.len(), 3, "{error}");
    let mut sorted = lines.clone();
    sorted.sort();
    assert_eq!(lines, sorted, "{error}");
    assert!(error.contains("options.actions[].uses"), "{error}");
    assert!(error.contains("options.actions[].ban"), "{error}");
    assert!(error.contains("options.actions[].require"), "{error}");
}

#[test]
fn repeated_require_and_blank_ban_entries_merge() {
    let yaml = r#"
actions:
  - uses: example/publish-widget
    ban: ["", overwrite, overwrite]
    require:
      keep-days: 1
  - uses: example/publish-widget/
    require:
      keep-days: 1
"#;
    let compiled = policy::compile(serde_yaml::from_str(yaml).unwrap()).unwrap();
    assert_eq!(compiled.actions.len(), 1);
    let source = "jobs:\n  build:\n    steps:\n      - uses: example/publish-widget@v9\n";
    let findings = parsed(source, yaml);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("without required input `keep-days`"));
}

#[test]
fn non_literal_inputs_and_unstructured_steps_are_judged() {
    let source = r#"
jobs:
  build:
    steps:
      - uses: example/publish-widget@v1
        with:
          keep-days: [1]
          replace: true
      - not-a-step
  other: plain
"#;
    let findings = parsed(source, POLICY);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("non-literal"), "{findings:?}");
}

#[test]
fn null_blank_and_unstructured_workflow_shapes_stay_precise() {
    let options = r#"
actions:
  - uses: example/publish-widget
    require:
      keep-days: ""
      replace: true
  - uses: example/ship-items
    ban: [overwrite]
"#;
    let source = r#"
jobs:
  1:
    steps:
      - uses: example/publish-widget@v1
        with:
          keep-days: 9
          replace: false
  build:
    runs-on: ubuntu-latest
  pack:
    steps:
      - run: echo items
      - name: "   "
        id: widget-step
        uses: example/publish-widget@v1
        with: "nope"
      - id: ship-step
        uses: example/ship-items@v1
        with:
          overwrite: null
          name: items
      - uses: example/publish-widget@v1
        with:
          keep-days:
            - 1
          replace: true
"#;
    let findings = parsed(source, options);
    assert_eq!(findings.len(), 3, "{findings:?}");
    assert!(findings.iter().any(|finding| {
        finding.message.contains("widget-step") && finding.message.contains("to \"\"")
    }));
    assert!(findings.iter().any(|finding| {
        finding.message.contains("widget-step") && finding.message.contains("replace")
    }));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("non-literal")));
    assert!(findings
        .iter()
        .all(|finding| !finding.message.contains("overwrite")));
}

#[test]
fn absent_jobs_and_unknown_job_lines_stay_inert() {
    assert!(parsed("name: Widgets\n", POLICY).is_empty());
    assert_eq!(
        super::lines::input_line("name: Widgets\n", "build", 0, "uses"),
        1
    );
    assert_eq!(
        super::lines::input_line(
            "jobs:\n  build:\n    runs-on: ubuntu-latest\n",
            "build",
            0,
            "uses"
        ),
        1
    );
}

#[test]
fn empty_file_list_is_silent() {
    let (root, _) = workflow("violations");
    assert!(check_with_files(&root, &load_config("policy.yml"), &[])
        .unwrap()
        .is_empty());
}

#[test]
fn collect_config_errors_reports_filter_include_and_schema_errors() {
    let config: NoMistakesConfig = serde_yaml::from_str(
        r#"
rules:
  - rule: github-actions-action-input-policy
    scope: repository
    include: ["[unclosed"]
    options:
      include: ["[bad-glob"]
  - rule: github-actions-action-input-policy
    scope: repository
    options:
      actions: true
"#,
    )
    .unwrap();
    let mut errors = std::collections::BTreeSet::new();
    super::collect_config_errors(std::path::Path::new("/repo"), &config, &mut errors);
    let joined = errors.into_iter().collect::<Vec<_>>().join("\n");
    assert!(joined.contains("[unclosed"), "{joined}");
    assert!(joined.contains("options.include"), "{joined}");
    assert!(joined.contains("options.actions"), "{joined}");
}
