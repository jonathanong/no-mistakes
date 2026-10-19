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

#[test]
fn saved_fail_fixtures_report_unverified_downloads() {
    let (root, config, files) = load_fixture("fail");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    let messages: Vec<_> = findings
        .iter()
        .map(|finding| finding.message.clone())
        .collect();
    assert!(
        messages.iter().any(|message| message.contains("a.tgz")),
        "{messages:?}"
    );
    assert!(
        messages.iter().any(|message| message.contains("b.tgz")),
        "{messages:?}"
    );
    assert!(
        messages.iter().any(|message| message.contains("c.tgz")),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("compact.tgz")),
        "{messages:?}"
    );
    assert!(
        messages.iter().any(|message| message.contains("tool.tgz")),
        "{messages:?}"
    );
    for url in [
        "tool-$VERSION.tgz",
        "tool-${VERSION}",
        "${{ inputs.version }}",
        "$(uname -m)",
        "sudo.tgz",
        "envcmd.tgz",
        "then.sh",
        "subshell.sh",
        "brace.sh",
        "bang.tgz",
        "after-heredoc.tgz",
        "herestring.tgz",
        "shift.tgz",
        "comment-apostrophe.tgz",
        "after-quote.tgz",
        "trailing-comment.tgz",
        "after-trailing.tgz",
        "comment-shift.tgz",
        "after-arith.tgz",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(url)),
            "{url} missing from {messages:?}"
        );
    }
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("api.github.com")),
        "{messages:?}"
    );
    assert!(
        messages.iter().all(|message| !message.contains(".sha256")),
        "{messages:?}"
    );
}

#[test]
fn option_fixtures_replace_hosts_and_add_helpers() {
    let (root, config, files) = load_fixture("helpers");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("ci_download_to"),
        "{findings:?}"
    );
    assert!(
        findings[0].message.contains("example.invalid"),
        "{findings:?}"
    );

    let (root, config, files) = load_fixture("empty-hosts");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("api.github.com")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("127.0.0.1")),
        "{findings:?}"
    );
}

#[test]
fn saved_pass_and_abstain_fixtures_are_silent() {
    for scenario in ["pass", "abstain"] {
        let (root, config, files) = load_fixture(scenario);
        let findings = super::check_with_files(&root, &config, &files).unwrap();
        assert!(findings.is_empty(), "{scenario}: {findings:?}");
    }
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

fn download_url(message: &str) -> &str {
    let marker = "downloads `";
    let start = message.find(marker).unwrap_or(0) + marker.len();
    let rest = message.get(start..).unwrap_or(message);
    let end = rest.find('`').unwrap_or(rest.len());
    &rest[..end]
}

fn target_for<'a>(findings: &'a [RuleFinding], url: &str) -> &'a str {
    findings
        .iter()
        .find(|finding| download_url(&finding.message) == url)
        .and_then(|finding| finding.target.as_deref())
        .unwrap_or("")
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
    let (root, _, files) = load_fixture("fail");
    let config = config_with_options(
        "trustedHosts:\n  - 'Example.Invalid:443'\n  - '[::1]'\ndownloadHelpers: [ci_fetch]\n",
    );
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("api.github.com"),
        "{findings:?}"
    );
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

#[test]
fn edge_fixtures_cover_unresolved_shell_and_job_shapes() {
    let (root, config, files) = load_fixture("edges");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    let mut urls: Vec<&str> = findings
        .iter()
        .map(|finding| download_url(&finding.message))
        .collect();
    let mut expected = vec![
        "https://[::1]",
        "https://[::1]:9/v6.tgz",
        "https://user:pw@example.invalid/auth.tgz",
        "https://example.invalid/bad-marker.tgz",
        "https://example.invalid/bare.tgz",
        "https://example.invalid/bool.tgz",
        "https://example.invalid/commented.tgz",
        "https://example.invalid/complex.tgz",
        "https://example.invalid/continued.tgz",
        "https://example.invalid/dashed.tgz",
        "https://example.invalid/dynamic.tgz",
        "https://example.invalid/empty-assign.tgz",
        "https://example.invalid/file{1}.sha256",
        "https://example.invalid/flow.tgz",
        "https://example.invalid/inline.tgz",
        "https://example.invalid/kept-after.tgz",
        "https://example.invalid/no-reason.tgz",
        "https://example.invalid/num.tgz",
        "https://example.invalid/quoted-nl.tgz",
        "https://example.invalid/spaced-assign.tgz",
        "https://example.invalid/stuck-skip.tgz",
        "https://example.invalid/trailing-junk.tgz",
        "https://example.invalid/unclosed.tgz",
    ];
    urls.sort_unstable();
    expected.sort_unstable();
    assert_eq!(urls, expected);

    let flow = findings
        .iter()
        .find(|finding| finding.file.ends_with("flow.yml"))
        .unwrap();
    assert_eq!(flow.line, 1, "{flow:?}");
    let num = findings
        .iter()
        .find(|finding| download_url(&finding.message).ends_with("num.tgz"))
        .unwrap();
    assert_ne!(num.line, 1, "{num:?}");
    assert_eq!(
        target_for(&findings, "https://example.invalid/num.tgz"),
        "7"
    );
    assert_eq!(
        target_for(&findings, "https://example.invalid/bool.tgz"),
        "true"
    );
    assert_eq!(
        target_for(&findings, "https://example.invalid/complex.tgz"),
        "job"
    );
}
