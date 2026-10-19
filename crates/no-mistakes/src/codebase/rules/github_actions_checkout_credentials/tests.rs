use super::checkout::check_source;
use super::{gha_kind, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

const WORKFLOW: &str = ".github/workflows/ci.yml";
const ACTION: &str = ".github/actions/local/action.yml";

fn check(rel: &str, source: &str) -> Vec<RuleFinding> {
    check_source(rel, source)
}

fn step_of(message: &str) -> usize {
    message
        .split("step ")
        .nth(1)
        .and_then(|rest| rest.split([' ', '`']).next())
        .and_then(|step| step.parse().ok())
        .unwrap_or(0)
}

fn identity(finding: &RuleFinding) -> String {
    let file = Path::new(&finding.file)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    format!(
        "{file}:{}:{}",
        finding.target.as_deref().unwrap_or(""),
        step_of(&finding.message)
    )
}

fn fixture_root(scenario: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-checkout-credentials/fixture")
            .join(scenario),
    )
}

fn fixture_files(scenario: &str) -> &'static [&'static str] {
    match scenario {
        "fail" => &[
            ".github/workflows/ci.yml",
            ".github/workflows/compact.yml",
            ".github/workflows/edges.yml",
            ".github/workflows/empty-job.yml",
            ".github/workflows/inherited.yml",
            ".github/workflows/job-overrides-read.yml",
            ".github/workflows/scopes.yml",
            ".github/workflows/shape.yml",
            ".no-mistakes.yml",
        ],
        "pass" => &[
            ".github/workflows/ci.yaml",
            ".github/workflows/empty-job.yml",
            ".github/workflows/override.yml",
            ".github/workflows/reusable-empty.yml",
            ".github/workflows/reusable-read.yml",
            ".no-mistakes.yml",
        ],
        "abstain" => &[
            ".github/actions/local/action.yml",
            ".github/actions/node/action.yaml",
            ".github/workflows/broken.yml",
            ".github/workflows/callable-job-write.yml",
            ".github/workflows/callable-write.yml",
            ".github/workflows/ci.yml",
            ".github/workflows/job-expression.yml",
            ".github/workflows/mapping-on.yml",
            ".github/workflows/nested/old.yml",
            ".github/workflows/null-permissions-call.yml",
            ".github/workflows/null-permissions.yml",
            ".github/workflows/pull-request-target.yml",
            ".github/workflows/reusable-list.yml",
            ".github/workflows/reusable-push.yml",
            ".github/workflows/reusable.yml",
            ".github/workflows/text.yml",
            ".no-mistakes.yml",
        ],
        "flow" => &[".github/workflows/ci.yml", ".no-mistakes.yml"],
        "suppression" => &[
            ".github/workflows/ci.yml",
            ".github/workflows/compact.yml",
            ".github/workflows/disabled.yml",
            ".no-mistakes.yml",
        ],
        other => panic!("unknown fixture scenario {other}"),
    }
}

fn load_fixture(scenario: &str) -> (PathBuf, NoMistakesConfig, Vec<PathBuf>) {
    let root = fixture_root(scenario);
    let config =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let files = fixture_files(scenario)
        .iter()
        .map(|name| root.join(name))
        .collect();
    (root, config, files)
}

fn enabled_config() -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    });
    config
}

#[test]
fn yaml_booleans_are_only_true_and_false() {
    let value: serde_yaml::Value = serde_yaml::from_str(
        "on: workflow_call\nno: no\nyes: yes\noff: off\nflag: TRUE\nallow: FALSE\n",
    )
    .unwrap();
    let map = value.as_mapping().unwrap();
    let key = |name: &str| serde_yaml::Value::String(name.to_string());
    assert!(
        map.contains_key(key("on")),
        "on must stay a string key: {value:?}"
    );
    assert_eq!(map[&key("no")].as_str(), Some("no"));
    assert_eq!(map[&key("yes")].as_str(), Some("yes"));
    assert_eq!(map[&key("off")].as_str(), Some("off"));
    assert_eq!(map[&key("flag")].as_bool(), Some(true));
    assert_eq!(map[&key("allow")].as_bool(), Some(false));
}

#[test]
fn kinds_and_missing_sources_are_skipped() {
    assert_eq!(gha_kind(".github/workflows/ci.yml"), Some(false));
    // GitHub does not run workflow files in subdirectories.
    assert!(gha_kind(".github/workflows/nested/ci.yaml").is_none());
    assert_eq!(gha_kind(ACTION), Some(true));
    assert_eq!(gha_kind(".github/actions/node/action.yaml"), Some(true));
    assert!(gha_kind(".github/workflows/ci.json").is_none());
    assert!(gha_kind(".github/actions/build/ci.yml").is_none());
    assert!(gha_kind("docs/ci.yml").is_none());
    assert!(check("README.md", "uses: actions/checkout@v4\n").is_empty());
    assert!(check(ACTION, "on: [:").is_empty());
    assert!(check(
        ACTION,
        "permissions:\n  contents: write\non: workflow_call\nruns:\n  using: composite\n  steps:\n    - uses: actions/checkout@v4\n",
    )
    .is_empty());
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let root = Path::new("/missing");
    assert!(super::check_file(Path::new("README.md"), root, &sources).is_empty());
    assert!(super::check_file(Path::new(WORKFLOW), root, &sources).is_empty());
}

#[test]
fn omitted_options_run_and_unknown_options_are_rejected() {
    assert!(
        super::check_with_files(Path::new("."), &enabled_config(), &[])
            .unwrap()
            .is_empty()
    );
    let mut config = enabled_config();
    config.rules[0].options = serde_yaml::from_str("nope: true").unwrap();
    let error = super::check_with_files(Path::new("."), &config, &[]).unwrap_err();
    assert!(error.to_string().contains("invalid options"), "{error}");
}

#[test]
fn unproven_or_structural_documents_do_not_flag() {
    let silent = [
        "name: ci\n",
        "- just\n",
        "on: push\njobs: []\n",
        "on: push\njobs:\n  build: hello\n",
        "on: push\njobs:\n  build:\n    permissions:\n      contents: write\n    steps: {}\n",
        "on: 1\njobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n",
        "on: true\njobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n",
        "on:\njobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n",
    ];
    for source in silent {
        assert!(check(WORKFLOW, source).is_empty(), "{source}");
    }
}

#[test]
fn proven_checkouts_keep_step_identity_and_real_uses_lines() {
    let source = "\
name: ci
on: push
uses: actions/checkout@v4
jobs:
  build:
    permissions:
      contents: write
    steps:
      - uses: 4
      - # kept
        uses: actions/setup-node@v4
      - uses: \"plain\"
      - run: |
          echo uses: actions/checkout@v4
      - uses: actions/checkout@v4
";
    let findings = check(WORKFLOW, source);
    assert_eq!(findings.len(), 1, "{findings:?}");
    let line = source
        .lines()
        .position(|line| line.trim_start().starts_with("- uses: actions/checkout@v4"))
        .unwrap()
        + 1;
    assert_eq!(findings[0].line, line);
    assert_eq!(findings[0].target.as_deref(), Some("build"));
    assert!(findings[0].message.contains("step 5"));
    assert!(findings[0]
        .message
        .contains("persist-credentials is not false"));
    assert!(findings[0].message.contains("actions/checkout"));
    assert_eq!(findings[0].rule, RULE_ID);

    let bom = format!("\u{feff}{source}");
    let bom_findings = check(WORKFLOW, &bom);
    assert_eq!(bom_findings.len(), 1, "{bom_findings:?}");
    assert_eq!(bom_findings[0].target.as_deref(), Some("build"));

    let flow = "\
on: push
permissions: {contents: write}
jobs: {build: {steps: [{uses: actions/checkout@v4}, {uses: actions/checkout@v4}]}}
";
    let flow_findings = check(WORKFLOW, flow);
    assert_eq!(flow_findings.len(), 2, "{flow_findings:?}");
    assert!(flow_findings.iter().all(|finding| finding.line == 1));
    assert!(flow_findings[0].message.contains("step 1"));
    assert!(flow_findings[1].message.contains("step 2"));

    let braced = "\
on: push
jobs:
  build:
    permissions:
      contents: write
    steps:
      - {uses: actions/checkout@v4}
";
    let braced_findings = check(WORKFLOW, braced);
    assert_eq!(braced_findings.len(), 1, "{braced_findings:?}");
    assert_eq!(braced_findings[0].line, 1);
}

#[test]
fn saved_fixtures_flag_only_proven_persisted_write_tokens() {
    let (root, config, files) = load_fixture("pass");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");

    let (root, config, files) = load_fixture("abstain");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");

    let (root, config, files) = load_fixture("fail");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    let mut got: Vec<String> = findings.iter().map(identity).collect();
    got.sort();
    let mut expected = Vec::new();
    for (job, step) in [
        ("omitted", 1),
        ("bool-true", 1),
        ("string-true", 1),
        ("string-upper", 1),
        ("yaml-true", 1),
        ("write-all", 1),
        ("spaced-write-all", 1),
        ("packages-first", 1),
        ("id-and-contents", 1),
        ("slash", 1),
        ("quoted", 1),
        ("quoted-spaces", 1),
        ("quoted-hash", 1),
        ("quoted-then-hash", 1),
        ("single-quoted", 1),
        ("single-hash", 1),
        ("commented", 1),
        ("after-script", 2),
        ("folded", 2),
        ("bare", 1),
        ("pinned", 1),
        ("no-ref", 1),
        ("trailing-slash", 1),
        ("sibling", 1),
        ("numeric-key", 1),
        ("spaced-access", 1),
        ("null-with", 1),
        ("empty-with", 1),
        ("string-step", 2),
        ("uses-mapping", 2),
        ("setup-then", 2),
        ("second-step", 2),
        ("two-flagged", 1),
        ("two-flagged", 2),
        ("extra-indent", 1),
    ] {
        expected.push(format!("ci.yml:{job}:{step}"));
    }
    for scope in [
        "actions",
        "artifact-metadata",
        "attestations",
        "checks",
        "code-quality",
        "contents",
        "copilot-requests",
        "deployments",
        "discussions",
        "issues",
        "packages",
        "pages",
        "pull-requests",
        "security-events",
        "statuses",
    ] {
        expected.push(format!("scopes.yml:{scope}:1"));
    }
    for (file, target) in [
        ("inherited.yml", "build"),
        ("job-overrides-read.yml", "build"),
        ("edges.yml", "9"),
        ("edges.yml", "true"),
        ("empty-job.yml", "job"),
        ("shape.yml", "folded"),
        ("shape.yml", "mixed"),
        ("shape.yml", "capitalized"),
    ] {
        expected.push(format!("{file}:{target}:1"));
    }
    for (file, target, step) in [
        ("compact.yml", "a", 2),
        ("compact.yml", "b", 2),
        ("shape.yml", "quoted-trap", 2),
        ("shape.yml", "mixed", 2),
    ] {
        expected.push(format!("{file}:{target}:{step}"));
    }
    expected.sort();
    assert_eq!(got, expected, "{findings:?}");

    let message = |file: &str, target: &str| {
        findings
            .iter()
            .find(|finding| {
                finding.file.ends_with(file) && finding.target.as_deref() == Some(target)
            })
            .unwrap_or_else(|| panic!("missing {file} {target}"))
            .message
            .clone()
    };
    assert!(message("ci.yml", "packages-first").contains("job permissions grant packages: write"));
    assert!(message("ci.yml", "write-all").contains("job permissions grant write-all"));
    assert!(message("ci.yml", "id-and-contents").contains("job permissions grant contents: write"));
    assert!(
        message("inherited.yml", "build").contains("workflow permissions grant contents: write")
    );
    assert!(message("job-overrides-read.yml", "build")
        .contains("job permissions grant contents: write"));
    let empty = findings
        .iter()
        .find(|finding| finding.file.ends_with("empty-job.yml"))
        .unwrap();
    assert_eq!(empty.line, 1);
    for finding in &findings {
        if finding.line == 1 {
            let empty_job = finding.file.ends_with("empty-job.yml");
            let flow_step = finding.file.ends_with("shape.yml")
                && finding.target.as_deref() == Some("mixed")
                && step_of(&finding.message) == 1;
            assert!(empty_job || flow_step, "unexpected line 1: {finding:?}");
            continue;
        }
        let text = std::fs::read_to_string(root.join(&finding.file)).unwrap();
        let line = text.lines().nth(finding.line - 1).unwrap();
        assert!(
            line.contains("uses:")
                && (line.contains("actions/checkout")
                    || line.contains("Actions/Checkout")
                    || line.contains("uses: >-"))
                && !line.trim_start().starts_with("echo"),
            "{}:{} `{line}`",
            finding.file,
            finding.line
        );
    }
    let compact = std::fs::read_to_string(root.join(".github/workflows/compact.yml")).unwrap();
    let shape = std::fs::read_to_string(root.join(".github/workflows/shape.yml")).unwrap();
    let located = |file: &str, job: &str, step: usize| {
        findings
            .iter()
            .find(|finding| {
                finding.file.ends_with(file)
                    && finding.target.as_deref() == Some(job)
                    && step_of(&finding.message) == step
            })
            .unwrap_or_else(|| panic!("missing {file} {job} step {step}"))
            .line
    };
    fn nth_line(text: &str, line: usize) -> &str {
        text.lines().nth(line - 1).expect("line")
    }
    assert!(nth_line(&compact, located("compact.yml", "a", 2)).starts_with("    - uses:"));
    assert!(nth_line(&compact, located("compact.yml", "b", 2)).starts_with("      - uses:"));
    assert_eq!(
        nth_line(&shape, located("shape.yml", "quoted-trap", 2)).trim(),
        "- uses: actions/checkout@v4"
    );
    assert!(nth_line(&shape, located("shape.yml", "folded", 1)).contains("uses: >-"));
    assert_eq!(
        nth_line(&shape, located("shape.yml", "mixed", 2)).trim(),
        "- uses: actions/checkout@v4"
    );
    assert!(
        nth_line(&shape, located("shape.yml", "capitalized", 1)).contains("Actions/Checkout@v4")
    );

    let (root, config, files) = load_fixture("flow");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].target.as_deref(), Some("build"));
    assert!(findings[0]
        .message
        .contains("workflow permissions grant contents: write"));
    let flow = std::fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();
    assert!(!flow.lines().next().unwrap().contains("actions/checkout"));
}

#[test]
fn suppression_directives_hide_file_line_and_next_line() {
    let (root, config, files) = load_fixture("suppression");
    let raw = super::check_with_files(&root, &config, &files).unwrap();
    assert_eq!(raw.len(), 6, "{raw:?}");
    let findings = crate::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    let ci = findings
        .iter()
        .find(|finding| finding.file.ends_with("ci.yml"))
        .unwrap();
    assert!(ci.message.contains("step 3"), "{ci:?}");
    let compact = findings
        .iter()
        .find(|finding| finding.file.ends_with("compact.yml"))
        .unwrap();
    assert_eq!(compact.target.as_deref(), Some("b"));
    assert!(compact.message.contains("step 2"), "{compact:?}");
    let text = std::fs::read_to_string(root.join(".github/workflows/compact.yml")).unwrap();
    let line = text.lines().nth(compact.line - 1).unwrap();
    assert!(line.contains("uses: actions/checkout@v4"), "{line}");
    assert!(!line.trim_start().starts_with("echo"), "{line}");
}

#[test]
fn compact_crlf_workflow_keeps_each_uses_line() {
    let source = "on: push
permissions: write-all
jobs:
  a:
    runs-on: ubuntu-latest
    steps:
    - run: echo hi
    - uses: actions/checkout@v4
  b:
    runs-on: ubuntu-latest
    steps:
      - run: echo
      - uses: actions/checkout@v4
"
    .replace('\n', "\r\n");
    let findings = check(WORKFLOW, &source);
    let mut expected: Vec<_> = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("uses: actions/checkout"))
        .map(|(index, _)| index + 1)
        .collect();
    let mut got: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    got.sort_unstable();
    expected.sort_unstable();
    assert_eq!(got, expected);
}
