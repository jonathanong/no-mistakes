use super::assign::expand_uppercase;
use super::judge::download_hits;
use super::url::{configured_host, http_url};
use super::verify::{step_verifies, takes_url_slot};
use super::words::{commands, is_assignment};
use super::RULE_ID;
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::Path;

#[test]
fn assignment_edges_leave_dynamic_values_alone() {
    assert!(expand_uppercase("BASE=\ncurl $BASE\n").contains("$BASE"));
    assert!(expand_uppercase("BASE=a b\n").contains("a b"));
    assert!(expand_uppercase("BASE=`x`\n").contains('`'));
    assert!(
        expand_uppercase("BASE='https://example.invalid' # pin\ncurl $BASE/a\n")
            .contains("https://example.invalid/a")
    );
    assert!(expand_uppercase("BASE=\"https://example.invalid\" trailing\n").contains("trailing"));
    assert!(expand_uppercase("curl $\n").contains('$'));
    assert!(expand_uppercase("curl ${UNCLOSED\n").contains("UNCLOSED"));
    let hits = download_hits(
        "curl -o https://example.invalid/out https://example.invalid/a.tgz\n",
        &[],
    );
    assert_eq!(hits.len(), 1);
    assert!(hits[0].url.ends_with("/a.tgz"));
}

#[test]
fn host_and_verifier_edges_match_the_documented_forms() {
    assert!(
        http_url("http://example.invalid/a.sha256sum")
            .unwrap()
            .checksum
    );
    assert!(
        http_url("https://example.invalid/a.sha512")
            .unwrap()
            .checksum
    );
    assert!(
        http_url("https://example.invalid/a.sha512sum")
            .unwrap()
            .checksum
    );
    assert!(
        http_url("https://example.invalid/SHA256SUMS")
            .unwrap()
            .checksum
    );
    assert!(
        http_url("https://example.invalid/SHA512SUMS")
            .unwrap()
            .checksum
    );
    assert!(
        http_url("https://example.invalid/SHA256SUMS?x=1")
            .unwrap()
            .checksum
    );
    assert_eq!(
        http_url("https://example.invalid").unwrap().host,
        "example.invalid"
    );
    assert_eq!(http_url("https://[::1]").unwrap().host, "::1");
    assert!(configured_host("host?x").is_none());
    assert!(configured_host("host#frag").is_none());
    assert!(configured_host("a b").is_none());
    assert!(configured_host("[::1").is_none());
    assert!(!super::url::has_skip(
        "# integrity-check: skipreason=vendor"
    ));
    assert!(!super::url::has_skip("# integrity-check: skip reason=.bad"));
    assert!(!super::url::has_skip("# not a marker"));
    assert!(!step_verifies("FOO=1"));
    assert!(!step_verifies("sha256sum -cx sums"));
    assert!(step_verifies("shasum -a 256 --check=sums"));
    assert!(takes_url_slot("curl", "--output"));
    assert!(!takes_url_slot("curl", "--"));
    assert!(!takes_url_slot("curl", "--output=out"));
    assert!(takes_url_slot("wget", "-O"));
    assert!(!is_assignment("nope"));
    let quoted = commands("echo \"a\nb\"");
    assert_eq!(quoted[0].words[1].text, "a\nb");
    let split = commands("echo a; echo b || echo c && echo d");
    assert!(split.len() >= 4);
}

#[test]
fn composite_and_helper_options_report_only_real_downloads() {
    let action = "\
name: fetch
runs:
  using: composite
  steps:
    - run: curl https://example.invalid/tool.tgz
";
    let findings = super::runs::check_source(
        ".github/actions/fetch/action.yml",
        action,
        &["example.invalid".to_string()],
        &[],
    );
    assert!(findings.is_empty(), "{findings:?}");
    let bare = "\
runs:
  using: composite
";
    assert!(
        super::runs::check_source(".github/actions/fetch/action.yml", bare, &[], &[]).is_empty()
    );
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("downloadHelpers: ['_ok-name']").unwrap(),
        ..Default::default()
    });
    assert!(super::check_with_files(Path::new("."), &config, &[])
        .unwrap()
        .is_empty());
}

fn check(source: &str) -> Vec<super::RuleFinding> {
    super::runs::check_source(
        ".github/workflows/ci.yml",
        source,
        &super::DEFAULT_TRUSTED_HOSTS
            .iter()
            .map(|host| (*host).to_string())
            .collect::<Vec<_>>(),
        &[],
    )
}

#[test]
fn workflow_shapes_that_are_not_downloads_stay_silent() {
    assert!(check(
        "\u{feff}name: bom\non: push\njobs:\n  build:\n    steps:\n      - run: echo ok\n"
    )
    .is_empty());
    assert!(check("on: push\njobs:\n  build: []\n").is_empty());
    assert!(check("on: push\njobs:\n  build:\n    steps:\n      - echo\n").is_empty());
    assert!(check("on: push\njobs: []\n").is_empty());
    let action = "\
name: node
runs:
  using: node20
  main: index.js
  steps:
    - run: curl https://example.invalid/a.tgz
";
    assert!(
        super::runs::check_source(".github/actions/node/action.yml", action, &[], &[]).is_empty()
    );
    let skipped = "\
on: push
jobs:
  build:
    steps:
      # integrity-check: skip reason=vendor
      - run: curl https://example.invalid/a.tgz
";
    assert!(check(skipped).is_empty());
    let flagged = check(
        "on: push\njobs:\n  true:\n    steps:\n      - run: curl https://example.invalid/a.tgz\n",
    );
    assert_eq!(flagged.len(), 1);
}
