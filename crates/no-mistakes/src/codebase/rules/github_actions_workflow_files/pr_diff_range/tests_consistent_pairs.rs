use super::options::{collect_config_errors, compile, Options};
use super::runs::check_source_with_options;
use super::RULE_ID;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

const WF: &str = ".github/workflows/ci.yml";

fn compile_yaml(yaml: &str) -> super::options::CompiledOptions {
    let opts: Options = serde_yaml::from_str(yaml).unwrap();
    compile(&opts).unwrap()
}

fn check(opts_yaml: &str, workflow_yaml: &str) -> (Vec<String>, Vec<String>) {
    let opts = compile_yaml(opts_yaml);
    let (findings, undecidable) = check_source_with_options(WF, workflow_yaml, &opts);
    (
        findings.into_iter().map(|f| f.message).collect(),
        undecidable.into_iter().map(|f| f.message).collect(),
    )
}

#[test]
fn consistent_pairs_allow_merge_commit_and_three_dot_pr_diffs() {
    let opts = "requireConsistentRevisionPairs: true";
    for workflow in [
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff HEAD^1 HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff ${{ github.event.pull_request.base.sha }}...${{ github.event.pull_request.head.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff origin/${{ github.base_ref }}...${{ github.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.pull_request.head.sha }}\n      - run: git diff ${{ github.event.pull_request.base.sha }}...HEAD\n",
        "on: merge_group\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.merge_group.base_sha }} ${{ github.event.merge_group.head_sha }}\n",
    ] {
        let (findings, undecidable) = check(opts, workflow);
        assert!(findings.is_empty(), "{workflow}\n{findings:?}");
        assert!(undecidable.is_empty(), "{workflow}\n{undecidable:?}");
    }
}

#[test]
fn consistent_pairs_reject_mismatched_base_merge_and_checkout_pairs() {
    let opts = "requireConsistentRevisionPairs: true";
    for workflow in [
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff ${{ github.event.pull_request.base.sha }}...${{ github.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff ${{ github.event.pull_request.base.sha }} HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff origin/${{ github.base_ref }} ${{ github.event.pull_request.head.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff HEAD^1 ${{ github.event.pull_request.head.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.pull_request.head.sha }}\n      - run: git diff HEAD^1 HEAD\n",
        "on: [pull_request, merge_group]\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.merge_group.base_sha }}...${{ github.event.pull_request.head.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.pull_request.base.sha }} origin/${{ github.base_ref }}\n",
    ] {
        let (findings, _) = check(opts, workflow);
        assert!(!findings.is_empty(), "expected mismatch for:\n{workflow}");
    }
}

#[test]
fn configured_env_input_and_flag_revision_pairs_catch_mismatches_and_unknowns() {
    let opts = r#"
revisionPairs:
  - baseEnv: DIFF_BASE
    headEnv: DIFF_HEAD
  - baseInput: base-rev
    headInput: head-rev
  - baseFlag: --base
    headFlag: --head
"#;
    let mismatched = r#"
on: pull_request
jobs:
  analyze:
    env:
      DIFF_BASE: ${{ github.event.pull_request.base.sha }}
      DIFF_HEAD: ${{ github.sha }}
    steps:
      - uses: actions/checkout@v4
      - uses: ./.github/actions/analyze
        with:
          base-rev: ${{ github.event.pull_request.base.sha }}
          head-rev: HEAD
      - run: node scripts/check.mjs --base=HEAD^1 --head=${{ github.event.pull_request.head.sha }}
"#;
    let (findings, undecidable) = check(opts, mismatched);
    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert!(undecidable.is_empty(), "{undecidable:#?}");

    let unknown = r#"
on: pull_request
jobs:
  analyze:
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ inputs.custom_ref }}
      - run: node scripts/check.mjs --base HEAD^1 --head HEAD
"#;
    let (findings, undecidable) = check(opts, unknown);
    assert!(findings.is_empty(), "{findings:#?}");
    assert_eq!(undecidable.len(), 1, "{undecidable:#?}");
}

#[test]
fn invalid_yaml_is_reported_as_unanalyzable_when_strict_enabled() {
    let (findings, undecidable) = check("requireConsistentRevisionPairs: true", "on: [");
    assert!(findings.is_empty());
    assert_eq!(undecidable.len(), 1);
    assert!(undecidable[0].contains("could not be parsed"));
}

#[test]
fn invalid_revision_pair_options_are_rejected() {
    for bad in [
        "unknownKey: true",
        "requireConsistentRevisionPairs: \"yes\"",
        "revisionPairs:\n  - baseEnv: BASE\n",
        "revisionPairs:\n  - baseEnv: BASE\n    headFlag: --head\n",
        "revisionPairs:\n  - baseEnv: BASE\n    headEnv: BASE\n",
        "revisionPairs:\n  - baseEnv: 123BAD\n    headEnv: HEAD\n",
        "revisionPairs:\n  - baseFlag: base\n    headFlag: --head\n",
    ] {
        let cfg = NoMistakesConfig {
            rules: vec![RuleDef {
                rule: RULE_ID.to_string(),
                scope: Some(RuleScope::Repository),
                options: serde_yaml::from_str(bad).unwrap(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut errors = Vec::new();
        collect_config_errors(Path::new("."), None, &cfg, &mut errors);
        assert!(!errors.is_empty(), "expected config error for:\n{bad}");
    }
}
