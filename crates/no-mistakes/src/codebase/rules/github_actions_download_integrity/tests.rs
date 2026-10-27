use super::runs::check_source;
use super::{gha_kind, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

const WORKFLOW: &str = ".github/workflows/ci.yml";

fn check(rel: &str, source: &str) -> Vec<RuleFinding> {
    check_source(rel, source, &default_hosts(), &[])
}

fn default_hosts() -> Vec<String> {
    super::DEFAULT_TRUSTED_HOSTS
        .iter()
        .map(|host| (*host).to_string())
        .collect()
}

fn fixture_root(scenario: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-download-integrity/fixture")
            .join(scenario),
    )
}

fn fixture_files(scenario: &str) -> &'static [&'static str] {
    match scenario {
        "fail" => &[
            ".github/actions/fetch/action.yml",
            ".github/workflows/ci.yml",
            ".github/workflows/compact.yml",
            ".no-mistakes.yml",
        ],
        "pass" => &[
            ".github/actions/fetch/action.yml",
            ".github/workflows/ci.yml",
            ".no-mistakes.yml",
        ],
        "suppression" => &[
            ".github/workflows/ci.yml",
            ".github/workflows/compact.yml",
            ".github/workflows/disabled.yml",
            ".no-mistakes.yml",
        ],
        "abstain" => &[
            ".github/actions/node/action.yaml",
            ".github/workflows/broken.yml",
            ".github/workflows/ci.yml",
            ".github/workflows/nested/old.yml",
            ".no-mistakes.yml",
        ],
        "helpers" => &[".github/workflows/ci.yml", ".no-mistakes.yml"],
        "empty-hosts" => &[".github/workflows/ci.yml", ".no-mistakes.yml"],
        "edges" => &[
            ".github/actions/fetch/action.yml",
            ".github/workflows/ci.yml",
            ".github/workflows/complex.yml",
            ".github/workflows/empty.yml",
            ".github/workflows/flow.yml",
            ".github/workflows/jobs.yml",
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
fn kinds_and_missing_sources_are_skipped() {
    assert_eq!(gha_kind(WORKFLOW), Some(false));
    assert!(gha_kind(".github/workflows/nested/ci.yaml").is_none());
    assert_eq!(gha_kind(".github/actions/fetch/action.yml"), Some(true));
    assert!(gha_kind("docs/ci.yml").is_none());
    assert!(check("README.md", "run: curl https://example.invalid/a.tgz\n").is_empty());
    assert!(check(WORKFLOW, "jobs: [\n").is_empty());
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
fn unverified_downloads_name_the_url_and_the_fix() {
    let source = "\
on: push
jobs:
  build:
    steps:
      - run: curl -fsSL https://example.invalid/a.tgz -o a.tgz
";
    let findings = check(WORKFLOW, source);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("https://example.invalid/a.tgz"));
    assert!(findings[0].message.contains("can change between runs"));
    assert!(findings[0].message.contains("sha256sum -c"));
    assert!(findings[0]
        .message
        .contains("integrity-check: skip reason=<slug>"));
    assert_eq!(findings[0].line, 5);
}

#[test]
fn trusted_hosts_checksums_verifiers_and_helpers() {
    let source = "\
on: push
jobs:
  build:
    steps:
      - run: |
          BASE_URL=\"https://example.invalid\"
          curl -fsSL \"$BASE_URL/b.tgz\" -o b.tgz
      - run: ci_download_to out https://example.invalid/tool.tgz
";
    let hosts = default_hosts();
    let plain = check_source(WORKFLOW, source, &hosts, &[]);
    assert_eq!(plain.len(), 1, "{plain:?}");
    assert!(
        plain[0].message.contains("https://example.invalid/b.tgz"),
        "{plain:?}"
    );
    let helped = check_source(WORKFLOW, source, &hosts, &["ci_download_to".to_string()]);
    assert_eq!(helped.len(), 2, "{helped:?}");
    let replaced = check_source(WORKFLOW, source, &["example.invalid".to_string()], &[]);
    assert!(replaced.is_empty(), "{replaced:?}");
}

// Output cases moved to cli_gha_snapshot_download_integrity.

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
    assert!(ci.message.contains("step 2"), "{ci:?}");
    let compact = findings
        .iter()
        .find(|finding| finding.file.ends_with("compact.yml"))
        .unwrap();
    assert_eq!(compact.target.as_deref(), Some("b"));
    assert!(compact.message.contains("step 3"), "{compact:?}");
    let text = std::fs::read_to_string(root.join(".github/workflows/compact.yml")).unwrap();
    let hidden = text.lines().nth(8).unwrap();
    assert!(hidden.contains("hidden.tgz"), "{hidden}");
    let kept = text.lines().nth(compact.line - 1).unwrap();
    assert!(kept.contains("kept.tgz"), "{kept}");
}

fn config_with_options(options: &str) -> NoMistakesConfig {
    let mut config = enabled_config();
    config.rules[0].options = serde_yaml::from_str(options).unwrap();
    config
}

#[test]
fn bad_host_and_helper_options_name_the_entry() {
    let host = super::check_with_files(
        Path::new("."),
        &config_with_options("trustedHosts: ['https://example.invalid']"),
        &[],
    )
    .unwrap_err();
    assert!(
        host.to_string()
            .contains("trustedHosts entry `https://example.invalid` is not a host"),
        "{host}"
    );
    let bracket = super::check_with_files(
        Path::new("."),
        &config_with_options("trustedHosts: ['[::1']"),
        &[],
    )
    .unwrap_err();
    assert!(
        bracket
            .to_string()
            .contains("trustedHosts entry `[::1` is not a host"),
        "{bracket}"
    );
    let helper = super::check_with_files(
        Path::new("."),
        &config_with_options("downloadHelpers: ['9bad']"),
        &[],
    )
    .unwrap_err();
    assert!(
        helper
            .to_string()
            .contains("downloadHelpers entry `9bad` is not a command name"),
        "{helper}"
    );
}

#[test]
fn port_and_bracket_hosts_replace_the_defaults() {
    // Output cases moved to cli_gha_snapshot_download_integrity.
    let silenced = check_source(
        WORKFLOW,
        "on: push\njobs:\n  build:\n    steps:\n      - run: 'curl -fsSL https://[::1]/silent.tgz'\n",
        &["::1".to_string()],
        &[],
    );
    assert!(silenced.is_empty(), "{silenced:?}");
}

#[test]
fn missing_workflow_source_is_skipped() {
    let root = fixture_root("pass");
    let missing = root.join(".github/workflows/missing.yml");
    assert!(!missing.exists());
    let findings = super::check_with_files(&root, &enabled_config(), &[missing]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn step_item_line_is_the_dash_not_the_run_key() {
    let source = "\
on: push
jobs:
  build:
    steps:
    - run: curl -fsSL https://example.invalid/a.tgz
    - name: fetch
      run: curl -fsSL https://example.invalid/b.tgz
";
    let runs =
        crate::codebase::rules::github_actions_script_expressions::step_run_lines(source, false);
    assert_eq!(runs.len(), 2, "{source}");
    let lines: Vec<_> = source.lines().collect();
    assert!(
        lines[runs[0].item_line - 1]
            .trim_start()
            .starts_with("- run:"),
        "{}",
        lines[runs[0].item_line - 1]
    );
    assert!(
        lines[runs[1].item_line - 1]
            .trim_start()
            .starts_with("- name:"),
        "{}",
        lines[runs[1].item_line - 1]
    );
}
