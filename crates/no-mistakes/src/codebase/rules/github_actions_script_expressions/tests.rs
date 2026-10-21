use super::analyze::check_source;
use super::{compile_patterns, default_constant_contexts, Options, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

const WORKFLOW: &str = ".github/workflows/ci.yml";
const ACTION: &str = ".github/actions/build/action.yml";

fn check(rel: &str, source: &str) -> Vec<RuleFinding> {
    let patterns = compile_patterns(&Options::default()).unwrap();
    check_source(rel, source, &patterns)
}

fn targets(findings: &[RuleFinding]) -> Vec<&str> {
    findings
        .iter()
        .map(|finding| finding.target.as_deref().unwrap_or(""))
        .collect()
}

fn workflow(body: &str) -> String {
    format!("name: ci\non: push\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n{body}")
}

#[test]
fn omitted_contexts_use_defaults_and_empty_list_replaces_them() {
    let omitted: Options = serde_yaml::from_str("{}").unwrap();
    assert!(omitted.constant_contexts.is_none());
    let empty: Options = serde_yaml::from_str("constantContexts: []").unwrap();
    assert_eq!(empty.constant_contexts.as_deref(), Some([].as_slice()));
    assert_eq!(
        default_constant_contexts(),
        super::DEFAULT_CONSTANT_CONTEXTS
    );
}

#[test]
fn invalid_constant_context_regex_fails_the_check() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("constantContexts: ['(']").unwrap(),
        ..Default::default()
    });
    let error = super::check_with_files(Path::new("."), &config, &[]).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("constantContexts"), "{message}");
    assert!(message.contains("pattern `(`"), "{message}");
}

#[test]
fn default_constants_are_allowed_and_other_contexts_are_not() {
    let source = workflow(
        r#"
      - run: |
          echo "${{ github.repository }}"
          echo "${{ github.repository_owner }}"
          echo "${{ github.server_url }}"
          echo "${{ github.run_id }}"
          echo "${{ github.run_number }}"
          echo "${{ github.run_attempt }}"
          echo "${{ github.sha }}"
          echo "${{ github.event_name }}"
          echo "${{ github.workspace }}"
          echo "${{ runner.os }}"
          echo "${{ runner.arch }}"
          echo "${{ runner.temp }}"
          echo "${{ job.services.redis.ports['6379'] }}"
          echo "${{ job.services.my-db.ports["5432"] }}"
          echo "${{  github.sha  }}"
          echo "${{ github.ref }}"
          echo "${{ github.actor }}"
          echo "${{ github.repository_visibility }}"
          echo "${{ GitHub.sha }}"
          echo "${{ github.sha || 'unknown' }}"
          echo "${{ job.services.redis.ports[6379] }}"
          echo "${{ }}"
"#,
    );
    let findings = check(WORKFLOW, &source);
    assert_eq!(
        targets(&findings),
        vec![
            "github.ref",
            "github.actor",
            "github.repository_visibility",
            "GitHub.sha",
            "github.sha || 'unknown'",
            "job.services.redis.ports[6379]",
            "",
        ],
        "{findings:?}"
    );
    assert!(findings[0].message.contains("before the shell parses"));
    assert!(findings[0].message.contains("echo \"$VALUE\""));
}

#[test]
fn empty_allowlist_flags_constant_contexts() {
    let patterns = compile_patterns(&Options {
        constant_contexts: Some(Vec::new()),
    })
    .unwrap();
    let findings = check_source(
        WORKFLOW,
        &workflow("      - run: echo \"${{ github.sha }}\"\n"),
        &patterns,
    );
    assert_eq!(targets(&findings), vec!["github.sha"]);
}

#[test]
fn custom_allowlist_replaces_the_default() {
    let patterns = compile_patterns(&Options {
        constant_contexts: Some(vec![r"^github\.actor$".to_string()]),
    })
    .unwrap();
    let findings = check_source(
        WORKFLOW,
        &workflow("      - run: echo \"${{ github.actor }} ${{ github.sha }}\"\n"),
        &patterns,
    );
    assert_eq!(targets(&findings), vec!["github.sha"]);
    assert!(!findings[0].message.contains("github.actor }}` in"));
}

#[test]
fn env_with_and_non_script_inputs_are_not_script_bodies() {
    // Job-level `env`, reusable `with`, and `defaults.run` are not script bodies.
    let source = r#"
name: ci
on: push
env:
  EVENT: ${{ github.event.head_commit.message }}
jobs:
  call:
    uses: org/repo/.github/workflows/reuse.yml@de0fac2e4500dabe0009e67214ff5f5447ce83dd
    with:
      name: ${{ github.actor }}
  build:
    runs-on: ubuntu-latest
    env:
      EVENT: ${{ github.event.head_commit.message }}
    defaults:
      run:
        shell: bash
    steps:
      - name: pass through env
        env:
          TITLE: ${{ github.event.pull_request.title }}
          REF: ${{ github.ref }}
        run: echo "$TITLE"
      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd
        with:
          ref: ${{ github.ref }}
      - if: ${{ github.ref == 'refs/heads/main' }}
        run: echo "${{ github.sha }}"
      - uses: example/action@de0fac2e4500dabe0009e67214ff5f5447ce83dd
        with:
          script: console.log("${{ github.actor }}")
      - uses: actions/github-script/extra@v7
        with:
          script: console.log("${{ github.actor }}")
      - run: echo ok
        # ${{ github.ref }}
"#;
    let findings = check(WORKFLOW, source);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn github_script_flags_only_its_script_body() {
    let source = workflow(
        r#"
      - with:
          script: |
            return "${{ github.event.issue.title }}"
          github-token: ${{ secrets.GITHUB_TOKEN }}
        uses: "actions/github-script@v7" # movable tag
      - uses: 'actions/github-script/'
        env:
          ACTOR: ${{ github.actor }}
        with:
          script: return process.env.ACTOR + "${{ github.sha }}"
          github-token: ${{ secrets.GITHUB_TOKEN }}
"#,
    );
    let findings = check(WORKFLOW, &source);
    assert_eq!(
        targets(&findings),
        vec!["github.event.issue.title"],
        "{findings:?}"
    );
    assert!(findings[0].message.contains("before JavaScript parses"));
    assert!(findings[0].message.contains("process.env"));
    // workflow() is six header lines plus the raw string's leading blank line.
    assert_eq!(findings[0].line, 10);
}

#[test]
fn block_plain_folded_and_quoted_runs_keep_locations() {
    let source = workflow(
        r#"
      - run: |
          # ${{ github.ref }}
          echo "${{ github.sha }}"
      - run:
          # ${{ github.actor }}
          echo "${{ runner.os }}"
      - run: >
          echo "${{ github.event_name }}"
      - run: |-
          echo "${{ secrets.TOKEN }}"
      - run: |2
          echo "${{ inputs.name }}"
      - run: &cmd |
          echo "${{ vars.FOO }}"
      - run: "echo ${{ env.FOO }}"
      - run: "echo \
          ${{ steps.build.outputs.value }}"
"#,
    );
    let findings = check(WORKFLOW, &source);
    assert_eq!(
        targets(&findings),
        vec![
            "github.ref",
            "secrets.TOKEN",
            "inputs.name",
            "vars.FOO",
            "env.FOO",
            "steps.build.outputs.value",
        ],
        "{findings:?}"
    );
    // The literal-block comment is line 9: six header lines, a blank line, then `run: |`.
    assert_eq!(findings[0].line, 9);
}

#[test]
fn quote_aware_expression_end_does_not_split_on_quoted_braces() {
    let source =
        workflow("      - run: echo \"${{ contains(github.event.head_commit.message, '}}') }}\"\n");
    let findings = check(WORKFLOW, &source);
    assert_eq!(
        targets(&findings),
        vec!["contains(github.event.head_commit.message, '}}')"]
    );
    let escaped = super::expressions::expressions(
        "${{ contains(github.event.head_commit.message, 'a\\'b') }}",
    );
    assert_eq!(
        escaped,
        vec!["contains(github.event.head_commit.message, 'a\\'b')"]
    );
    assert!(super::expressions::expressions("echo ${{ github.ref").is_empty());
}

#[test]
fn same_column_steps_and_multiple_jobs_are_scanned() {
    let source = r#"
name: ci
on: push
jobs:
  one:
    steps:
    - run: echo ${{ github.ref }}
    - run: echo ${{ github.sha }}
  two:
    steps:
      - run: echo ${{ github.actor }}
"#;
    let findings = check(WORKFLOW, source);
    assert_eq!(
        targets(&findings),
        vec!["github.ref", "github.actor"],
        "{findings:?}"
    );
}

#[test]
fn composite_steps_are_scanned_and_other_actions_are_not() {
    let composite = r#"
name: Build
description: build
inputs:
  name:
    required: true
runs:
  using: composite
  steps:
    - shell: bash
      env:
        NAME: ${{ inputs.name }}
      run: echo "$NAME"
    - shell: bash
      run: echo "${{ inputs.name }}"
    - uses: actions/github-script@v7
      with:
        script: console.log(`${{ github.actor }}`)
"#;
    let findings = check(ACTION, composite);
    assert_eq!(
        targets(&findings),
        vec!["inputs.name", "github.actor"],
        "{findings:?}"
    );

    let docker = r#"
name: Image
runs:
  using: docker
  image: Dockerfile
  args:
    - ${{ github.ref }}
"#;
    assert!(check(".github/actions/image/action.yml", docker).is_empty());

    let node = r#"
name: Node
runs:
  using: node20
  main: index.js
  steps:
    - run: echo ${{ github.ref }}
"#;
    assert!(check(".github/actions/node/action.yml", node).is_empty());
    assert!(check(".github/actions/node/action.yaml", node).is_empty());
}

#[test]
fn anchors_flow_and_split_expressions_are_unanalyzable() {
    let alias = workflow("      - run: &cmd echo \"${{ github.ref }}\"\n      - run: *cmd\n");
    let findings = check(WORKFLOW, &alias);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("could not be aligned"),
        "{findings:?}"
    );
    assert!(findings[0].target.is_none());

    let flow = workflow("      - { run: \"echo ${{ github.ref }}\" }\n");
    let findings = check(WORKFLOW, &flow);
    assert!(
        findings[0].message.contains("could not be aligned"),
        "{findings:?}"
    );

    let split = workflow("      - run: |\n          echo ${{ github.ref\n          }}\n");
    let findings = check(WORKFLOW, &split);
    assert!(
        findings[0].message.contains("could not be aligned"),
        "{findings:?}"
    );
}

#[test]
fn sparse_steps_escaped_quotes_and_non_script_blocks_stay_aligned() {
    let source = workflow(
        r#"
      - "plain"
      -
      - # note
      - name: |
          title
        run: echo "say \"${{ github.ref }}\""
      - run:
          shell: bash
      - uses: actions/github-script@v7
        with:
          script: 1
      - run: "echo \
          still \
          ${{ github.actor }}"
"#,
    );
    let findings = check(WORKFLOW, &source);
    assert_eq!(
        targets(&findings),
        vec!["github.ref", "github.actor"],
        "{findings:?}"
    );
    assert!(check(WORKFLOW, "jobs: []\n").is_empty());
    assert!(check(
        ACTION,
        "name: Pack\nruns:\n  using: composite\n  steps: {}\n"
    )
    .is_empty());
}

#[test]
fn invalid_yaml_is_reported_and_not_guessed() {
    let findings = check(WORKFLOW, "jobs: [\n");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("invalid YAML"), "{findings:?}");
    assert!(findings[0].line >= 1);

    // serde_yaml accepts an empty or comment-only document as a null value.
    // There is no script body to judge, so this is not an invalid-YAML finding.
    assert!(check(WORKFLOW, "").is_empty());
    assert!(check(WORKFLOW, "# only a comment\n").is_empty());
}

#[test]
fn benign_workflow_shapes_pass() {
    assert!(check(WORKFLOW, "on: push\n").is_empty());
    assert!(check(WORKFLOW, "---\nname: ci\non: push\n").is_empty());
    let bom = format!(
        "\u{feff}{}",
        workflow("      - run: echo \"${{ github.sha }}\"\n")
    );
    assert!(check(WORKFLOW, &bom).is_empty(), "{bom}");
}

#[test]
fn non_workflow_paths_and_missing_sources_are_skipped() {
    let patterns = compile_patterns(&Options::default()).unwrap();
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let readme = Path::new("README.md");
    assert!(super::check_file(readme, Path::new("/missing"), &patterns, &sources).is_empty());
    let workflow = Path::new(".github/workflows/ci.yml");
    assert!(super::check_file(workflow, Path::new("/missing"), &patterns, &sources).is_empty());
    assert!(super::analyze::gha_kind("docs/readme.yml").is_none());
    assert!(super::analyze::gha_kind(".github/actions/build/action.md").is_none());
    assert_eq!(
        super::analyze::gha_kind(".github/workflows/nested/ci.yaml"),
        Some(false)
    );
}

fn fixture_root(scenario: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-script-expressions/fixture")
            .join(scenario),
    )
}

fn fixture_files(root: &Path, scenario: &str) -> Vec<PathBuf> {
    let names: &[&str] = match scenario {
        "pass" => &[
            ".github/actions/image/action.yml",
            ".github/actions/pack/action.yml",
            ".github/workflows/ci.yml",
            ".no-mistakes.yml",
        ],
        "fail" => &[
            ".github/actions/pack/action.yml",
            ".github/workflows/ci.yml",
            ".no-mistakes.yml",
        ],
        "allowlist" => &[".github/workflows/ci.yml", ".no-mistakes.yml"],
        "unanalyzable" => &[
            ".github/workflows/alias.yml",
            ".github/workflows/flow.yml",
            ".no-mistakes.yml",
        ],
        other => panic!("unknown fixture scenario {other}"),
    };
    names.iter().map(|name| root.join(name)).collect()
}

fn load_fixture(scenario: &str) -> (PathBuf, NoMistakesConfig, Vec<PathBuf>) {
    let root = fixture_root(scenario);
    let config =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let files = fixture_files(&root, scenario);
    (root, config, files)
}

#[test]
fn saved_fixtures_match_allowed_flagged_allowlist_and_suppression() {
    let (root, config, files) = load_fixture("pass");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");

    let (root, config, files) = load_fixture("fail");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    let mut got = targets(&findings);
    got.sort_unstable();
    assert_eq!(
        got,
        vec![
            "env.FOO",
            "github.actor",
            "github.actor",
            "github.actor",
            "github.event.head_commit.message",
            "github.event.issue.title",
            "github.ref",
            "github.sha || 'unknown'",
            "inputs.name",
            "inputs.name",
            "job.services.redis.ports[6379]",
            "secrets.TOKEN",
            "steps.id.outputs.value",
            "vars.FOO",
        ]
    );
    assert!(findings
        .iter()
        .any(|finding| finding.file.ends_with("action.yml")));
    assert!(findings
        .iter()
        .any(|finding| finding.file.ends_with("ci.yml")));

    let (root, config, files) = load_fixture("allowlist");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert_eq!(targets(&findings), vec!["github.sha"]);

    let root = fixture_root("suppression");
    let findings = crate::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert_eq!(
        targets(&findings),
        vec!["github.event.head_commit.message"],
        "{findings:?}"
    );

    let (root, config, files) = load_fixture("unanalyzable");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.len() == 2
            && findings
                .iter()
                .all(|finding| finding.message.contains("aligned")),
        "{findings:?}"
    );
}
