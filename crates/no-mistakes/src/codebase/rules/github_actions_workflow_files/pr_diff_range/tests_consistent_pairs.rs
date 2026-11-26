use super::options::{collect_config_errors, compile, Options};
use super::runs::check_source_with_options;
use super::RULE_ID;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

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
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n      - run: git diff HEAD^1 HEAD^2\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ''\n      - run: git diff HEAD^1 ${{ github.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: refs/pull/${{ github.event.pull_request.number }}/merge\n      - run: git diff HEAD~1 HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.sha }}\n      - run: git diff ${{ github.event.pull_request.base.sha }}...${{ github.event.pull_request.head.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff origin/${{ github.base_ref }}...${{ github.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.pull_request.head.sha }}\n      - run: git diff ${{ github.event.pull_request.base.sha }}...HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - run: git diff origin/${{ github.base_ref }}...${{ github.head_ref }}\n",
        "on: [pull_request, merge_group]\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.pull_request.base.sha || github.event.merge_group.base_sha }}...${{ github.event.pull_request.head.sha || github.event.merge_group.head_sha }}\n",
        "on: merge_group\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.merge_group.head_sha }}\n      - run: git diff ${{ github.event.merge_group.base_sha }} HEAD\n",
        "on: merge_group\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.merge_group.base_sha }} ${{ github.event.merge_group.head_sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - run: git diff foo bar\n",
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
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff origin/${{ github.base_ref }} ${{ github.sha }}\n",
        "on: [pull_request, push]\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff origin/${{ github.base_ref }}...${{ github.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff HEAD^1 ${{ github.event.pull_request.head.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.pull_request.head.sha }}\n      - run: git diff HEAD^1 HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.pull_request.base.sha }}\n      - run: git diff HEAD^1 HEAD\n",
        "on: [pull_request, merge_group]\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.merge_group.base_sha }}...${{ github.event.pull_request.head.sha }}\n",
        "on: [pull_request, merge_group]\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.pull_request.base.sha }}...${{ github.event.merge_group.head_sha }}\n",
        "on: [pull_request, merge_group]\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.merge_group.base_sha }} ${{ github.sha }}\n",
        "on: [pull_request, merge_group]\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.pull_request.base.sha || github.event.merge_group.base_sha }}...${{ github.event.pull_request.head.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.pull_request.base.sha }} origin/${{ github.base_ref }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.pull_request.head.sha }} ${{ github.sha }}\n",
    ] {
        let (findings, _) = check(opts, workflow);
        assert!(!findings.is_empty(), "expected mismatch for:\n{workflow}");
    }
}

#[test]
fn configured_env_input_and_flag_revision_pairs_cover_scopes_and_edge_shapes() {
    let opts = r#"
revisionPairs:
  - baseEnv: DIFF_BASE
    headEnv: DIFF_HEAD
  - baseInput: base-rev
    headInput: head-rev
  - baseFlag: --base
    headFlag: --head
"#;
    let workflow = r#"
on: pull_request
env:
  DIFF_BASE: ${{ github.event.pull_request.base.sha }}
  DIFF_HEAD: ${{ github.sha }}
jobs:
  caller:
    uses: ./.github/workflows/callee.yml
    with:
      base-rev: ${{ github.event.pull_request.head.sha }}
      head-rev: ${{ github.event.pull_request.base.sha }}
  analyze:
    steps:
      - uses: actions/checkout@v4
        with:
          ref:
      - env:
          DIFF_BASE: origin/${{ github.base_ref }}
          DIFF_HEAD: ${{ github.sha }}
        run: |
          node scripts/check.mjs --base=HEAD^1 --base=HEAD^1 --head=HEAD -- --base ignored
          node scripts/check.mjs --base HEAD^1 --base origin/${{ github.base_ref }} --head HEAD
          node scripts/check.mjs --base= --head HEAD
          node scripts/check.mjs --base ${{ github.event.pull_request.base.sha }} --head
      - uses: ./.github/actions/analyze
        with:
          base-rev: 123
          head-rev: HEAD
"#;
    let (findings, undecidable) = check(opts, workflow);
    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert_eq!(undecidable.len(), 4, "{undecidable:#?}");

    for unknown_checkout in [
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with: bad\n      - run: node x --base HEAD^1 --head HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: 123\n      - run: node x --base HEAD^1 --head HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          '<<': { ref: main }\n      - run: node x --base HEAD^1 --head HEAD\n",
    ] {
        let (findings, undecidable) = check(opts, unknown_checkout);
        assert!(findings.is_empty(), "{unknown_checkout}\n{findings:#?}");
        assert_eq!(undecidable.len(), 1, "{unknown_checkout}\n{undecidable:#?}");
    }
}

#[test]
fn invalid_yaml_and_file_suppression_follow_policy() {
    let (findings, undecidable) = check("requireConsistentRevisionPairs: true", "on: [");
    assert!(findings.is_empty());
    assert_eq!(undecidable.len(), 1);
    assert!(undecidable[0].contains("could not be parsed"));

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-pr-diff-range/fixture/suppression");
    let root = crate::codebase::ts_resolver::normalize_path(&base);
    let files = vec![
        root.join(".github/workflows/file-disabled.yml"),
        root.join(".github/workflows/suppressed.yml"),
    ];
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            ..Default::default()
        }],
        ..Default::default()
    };
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let out = super::diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
        .unwrap();
    assert_eq!(out.findings.len(), 1);
}

#[test]
fn invalid_revision_pair_options_are_rejected() {
    for bad in [
        "unknownKey: true",
        "requireConsistentRevisionPairs: \"yes\"",
        "revisionPairs:\n  - {}\n",
        "revisionPairs:\n  - baseEnv: BASE\n",
        "revisionPairs:\n  - headEnv: HEAD\n",
        "revisionPairs:\n  - baseEnv: '   '\n    headEnv: HEAD\n",
        "revisionPairs:\n  - baseEnv: BASE\n    headFlag: --head\n",
        "revisionPairs:\n  - baseEnv: BASE\n    headEnv: BASE\n",
        "revisionPairs:\n  - baseEnv: 123BAD\n    headEnv: HEAD\n",
        "revisionPairs:\n  - baseInput: base\n",
        "revisionPairs:\n  - baseFlag: base\n    headFlag: --head\n",
        "revisionPairs:\n  - baseFlag: '--a b'\n    headFlag: --head\n",
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
