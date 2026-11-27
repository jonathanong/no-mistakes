use super::super::options::{compile, CompiledOptions, Options};
use super::super::runs::check_source_with_options;
use super::super::RULE_ID;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

const WF: &str = ".github/workflows/ci.yml";

fn compile_yaml(yaml: &str) -> CompiledOptions {
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
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout/@v4\n      - run: git diff HEAD^1 HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n      - run: git diff HEAD HEAD^1\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n      - run: git diff HEAD^1 HEAD^2\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ''\n      - run: git diff HEAD^1 ${{ github.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: refs/pull/${{ github.event.pull_request.number }}/merge\n      - run: git diff HEAD~1 HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.sha }}\n      - run: git diff ${{ github.event.pull_request.base.sha }}...${{ github.event.pull_request.head.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.pull_request.head.sha }}...${{ github.event.pull_request.base.sha }}\n",
        "on: merge_group\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.merge_group.base_sha }} ${{ github.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff refs/remotes/origin/${{ github.base_ref }}...${{ github.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.pull_request.head.sha }}\n      - run: git diff ${{ github.event.pull_request.base.sha }}...HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.head_ref }}\n      - run: git diff origin/${{ github.event.pull_request.base.ref }}...HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - run: git diff origin/${{ github.base_ref }}...${{ github.event.pull_request.head.ref }}\n",
        "on: [pull_request, merge_group]\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.pull_request.base.sha || github.event.merge_group.base_sha }}...${{ github.event.pull_request.head.sha || github.event.merge_group.head_sha }}\n",
        "on: [pull_request, merge_group]\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.merge_group.base_sha || github.event.pull_request.base.sha }}...${{ github.event.merge_group.head_sha || github.event.pull_request.head.sha }}\n",
        "on: merge_group\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff HEAD^1 HEAD\n",
        "on: merge_group\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff HEAD^1 HEAD^2\n",
        "on: merge_group\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff HEAD^1 ${{ github.event.merge_group.head_sha }}\n",
        "on: merge_group\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff HEAD^1 ${{ github.sha }}\n",
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
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.pull_request.head.ref }}\n      - run: git diff HEAD^1 HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.base_ref }}\n      - run: git diff HEAD^1 HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.pull_request.base.ref }}\n      - run: git diff ${{ github.event.pull_request.base.sha }} HEAD\n",
        "on: [pull_request, merge_group]\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.merge_group.base_sha }}...${{ github.event.pull_request.head.sha }}\n",
        "on: [pull_request, merge_group]\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.pull_request.base.sha }}...${{ github.event.merge_group.head_sha }}\n",
        "on: [pull_request, merge_group]\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.merge_group.base_sha }} ${{ github.sha }}\n",
        "on: [pull_request, merge_group]\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.pull_request.base.sha || github.event.merge_group.base_sha }}...${{ github.event.pull_request.head.sha }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.pull_request.base.sha }} origin/${{ github.base_ref }}\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.pull_request.head.sha }} ${{ github.sha }}\n",
        "on: merge_group\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.merge_group.base_sha }} ${{ github.event.merge_group.base_sha }}\n",
        "on: merge_group\njobs:\n  a:\n    steps:\n      - run: git diff ${{ github.event.merge_group.head_sha }} HEAD\n",
        "on: merge_group\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - run: git diff HEAD^1 ${{ github.event.pull_request.head.sha }}\n",
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
      - uses: actions/setup-node@v4
      - uses: actions/checkout@v4
        with:
          ref:
      - env:
          DIFF_BASE: origin/${{ github.base_ref }}
        run: |
          node scripts/check.mjs --base-extra x --base=HEAD^1 --base=HEAD^1 --head=HEAD --head=HEAD -- --base ignored
          node scripts/check.mjs --base HEAD^1 --base origin/${{ github.base_ref }} --head HEAD
          node scripts/check.mjs --base HEAD^1 --head HEAD --head ${{ github.sha }}
          node scripts/check.mjs --base= --head HEAD
          node scripts/check.mjs --base ${{ github.event.pull_request.base.sha }} --head
          node scripts/check.mjs --base "$UNSET_BASE" --head HEAD
          node scripts/check.mjs --base ${{ github.event.pull_request.base.sha || github.sha }} --head HEAD
      - uses: ./.github/actions/analyze
        with:
          base-rev: 123
          head-rev: 456
"#;
    let (findings, undecidable) = check(opts, workflow);
    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert_eq!(undecidable.len(), 7, "{undecidable:#?}");

    let flag_only = "revisionPairs:\n  - baseFlag: --base\n    headFlag: --head\n";
    for unknown_checkout in [
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with: bad\n      - run: node x --base HEAD^1 --head HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: 123\n      - run: node x --base HEAD^1 --head HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: main\n      - run: node x --base HEAD^1 --head HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ inputs.ref }}\n      - run: node x --base HEAD^1 --head HEAD\n",
        "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          '<<': { ref: main }\n      - run: node x --base HEAD^1 --head HEAD\n",
    ] {
        let (findings, undecidable) = check(flag_only, unknown_checkout);
        assert!(findings.is_empty(), "{unknown_checkout}\n{findings:#?}");
        assert_eq!(undecidable.len(), 1, "{unknown_checkout}\n{undecidable:#?}");
    }

    let non_mapping = r#"
on: pull_request
env: ${{ inputs.workflow_env }}
jobs:
  caller:
    uses: ./.github/workflows/callee.yml
    with: ${{ inputs.caller_with }}
  job_shapes:
    env:
    steps:
      - env: bad
        uses: ./.github/actions/analyze
        with:
      - uses: ./.github/actions/analyze
        with:
          '<<': { base-rev: HEAD^1 }
"#;
    let (findings, undecidable) = check(opts, non_mapping);
    assert!(findings.is_empty(), "{findings:#?}");
    assert_eq!(undecidable.len(), 4, "{undecidable:#?}");
    assert!(undecidable[0].contains("workflow (env)"));
    assert!(undecidable[1].contains("job `caller` (with)"));
    assert!(undecidable[2].contains("job `job_shapes` step 1 (env)"));
    assert!(undecidable[3].contains("job `job_shapes` step 2 (with)"));

    let (findings, undecidable) = check(flag_only, non_mapping);
    assert!(findings.is_empty(), "{findings:#?}");
    assert!(undecidable.is_empty(), "{undecidable:#?}");
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
        root.join("README.md"),
        root.join(".github/workflows/missing.yml"),
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
    let sources = crate::codebase::rules::source_store_for_files(&files[2..]);
    let out =
        super::super::diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
            .unwrap();
    assert_eq!(out.findings.len(), 1);

    let dispatched = crate::codebase::rules::github_actions_workflow_files::dispatch::run(
        RULE_ID, &root, &config, &files, &sources, false,
    )
    .unwrap()
    .unwrap();
    assert_eq!(dispatched.len(), 1);

    let mut repo_policy = config.clone();
    repo_policy.unanalyzable = Some(crate::config::v2::Unanalyzable::Report);
    repo_policy.rules[0].options =
        serde_yaml::from_str("requireConsistentRevisionPairs: true").unwrap();
    let strict_root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-pr-diff-range/fixture/consistent-pairs"),
    );
    let strict_files = vec![strict_root.join(".github/workflows/unanalyzable.yml")];
    let strict_sources = crate::codebase::rules::source_store_for_files(&strict_files);
    let strict_out = super::super::diagnostics_with_files_and_sources(
        &strict_root,
        &repo_policy,
        &strict_files,
        &strict_sources,
        true,
    )
    .unwrap();
    assert_eq!(strict_out.findings.len(), 2);
    assert!(strict_out.advisories.is_empty());
}

#[test]
fn partial_pairs_and_missing_jobs_stay_quiet() {
    let env_only = "revisionPairs:\n  - baseEnv: DIFF_BASE\n    headEnv: DIFF_HEAD\n";
    let no_jobs =
        "on: pull_request\nenv:\n  DIFF_BASE: ${{ github.event.pull_request.base.sha }}\n";
    let (findings, undecidable) = check(env_only, no_jobs);
    assert!(findings.is_empty());
    assert!(undecidable.is_empty());

    let partial = r#"
on: pull_request
jobs:
  a:
    env:
      DIFF_BASE: ${{ github.event.pull_request.base.sha }}
    steps:
      - env:
          OTHER: 1
        uses: ./.github/actions/x
        with:
          base-rev: HEAD^1
      - run: echo ok
"#;
    let opts = "revisionPairs:\n  - baseEnv: DIFF_BASE\n    headEnv: DIFF_HEAD\n  - baseInput: base-rev\n    headInput: head-rev\n";
    let (findings, undecidable) = check(opts, partial);
    assert!(findings.is_empty());
    assert!(undecidable.is_empty());

    let bad_jobs = "on: pull_request\njobs: bad\n";
    let (findings, undecidable) = check(opts, bad_jobs);
    assert!(findings.is_empty());
    assert_eq!(undecidable.len(), 1, "{undecidable:#?}");
    assert!(undecidable[0].contains("workflow `jobs`"));

    let bad_shapes = r#"
on: pull_request
jobs:
  scalar_job: nope
  merged_job:
    '<<': { runs-on: ubuntu-latest }
  bad_steps:
    steps: nope
  step_shapes:
    steps:
      - nope
      - '<<': { run: echo ok }
"#;
    let (findings, undecidable) = check(opts, bad_shapes);
    assert!(findings.is_empty(), "{findings:#?}");
    assert_eq!(undecidable.len(), 5, "{undecidable:#?}");
    assert!(undecidable[0].contains("job `scalar_job`"));
    assert!(undecidable[1].contains("job `merged_job`"));
    assert!(undecidable[2].contains("job `bad_steps` `steps`"));
    assert!(undecidable[3].contains("job `step_shapes` step 1"));
    assert!(undecidable[4].contains("job `step_shapes` step 2"));
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
        "revisionPairs:\n  - baseEnv: A-B\n    headEnv: HEAD\n",
        "revisionPairs:\n  - baseEnv: BASE\n    headEnv: 123BAD\n",
        "revisionPairs:\n  - baseInput: base\n",
        "revisionPairs:\n  - headInput: head\n",
        "revisionPairs:\n  - baseInput: ' '\n    headInput: head\n",
        "revisionPairs:\n  - baseInput: base\n    headInput: ' '\n",
        "revisionPairs:\n  - baseFlag: --base\n",
        "revisionPairs:\n  - headFlag: --head\n",
        "revisionPairs:\n  - baseFlag: '-'\n    headFlag: --head\n",
        "revisionPairs:\n  - baseFlag: base\n    headFlag: --head\n",
        "revisionPairs:\n  - baseFlag: --base\n    headFlag: head\n",
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
        let result = crate::codebase::rules::validate_options::validate_rule_options(
            Path::new("."),
            Path::new(".no-mistakes.yml"),
            &cfg,
            &|_| None,
        );
        assert!(result.is_err(), "expected config error for:\n{bad}");
    }
}
