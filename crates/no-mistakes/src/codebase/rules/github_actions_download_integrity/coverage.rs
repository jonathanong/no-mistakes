use super::assign::expand_uppercase;
use super::judge::{self, configured_host, download_hits};
use super::url::{self, http_url};
use super::verify::{step_verifies, takes_url_slot};
use super::words::{self, commands, has_heredoc};
use super::RULE_ID;
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::Path;

fn workflow(script: &str) -> String {
    format!("on: push\njobs:\n  build:\n    steps:\n      - run: {script}\n")
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
fn shell_forms_cover_quotes_pipes_and_verifiers() {
    let expanded = expand_uppercase(
        "BASE_URL=\"https://example.invalid\" # pin\ncurl \"$BASE_URL/a.tgz\"\ncurl '$BASE_URL/b'\ncurl ${BASE_URL}/c\ncurl $OTHER\n",
    );
    assert!(expanded.contains("https://example.invalid/a.tgz"));
    assert!(expanded.contains("'$BASE_URL/b'"));
    assert!(expanded.contains("https://example.invalid/c"));
    assert!(expanded.contains("$OTHER"));
    assert_eq!(
        expand_uppercase("BASE_URL=\"https://a\"\nALT_URL=\"https://b\"\ncurl \"$BASE_URL/a\"\n"),
        "BASE_URL=\"https://a\"\nALT_URL=\"https://b\"\ncurl \"$BASE_URL/a\"\n"
    );
    assert!(expand_uppercase("BASE_URL=$HOME\ncurl \"$BASE_URL/a\"\n").contains("$BASE_URL"));
    assert!(expand_uppercase("BASE_URL=\"https://a\" extra\n").contains("extra"));
    assert!(expand_uppercase("BASE_URL=\"https://a\n").contains("BASE_URL"));
    assert!(expand_uppercase("base_url=\"https://a\"\n").starts_with("base_url"));
    assert!(
        expand_uppercase("URL=https://example.invalid/plain\ncurl $URL/a\n")
            .contains("https://example.invalid/plain/a")
    );

    assert!(has_heredoc("curl <<EOF\nhttps://example.invalid\nEOF\n"));
    assert!(!has_heredoc("echo '<<'\n"));
    assert!(check(&workflow("curl <<EOF")).is_empty());
    assert!(check(&workflow("curl `echo https://example.invalid/a`")).is_empty());
    assert!(check(&workflow("curl $(cat url)")).is_empty());
    assert_eq!(
        check(&workflow("curl https://example.invalid/a | wc")).len(),
        1
    );
    assert_eq!(
        check(&workflow("true && curl https://example.invalid/a")).len(),
        1
    );
    let continued = commands("curl https://example.invalid/a \\\n  -o a.tgz");
    assert_eq!(continued.len(), 1);
    assert!(continued[0]
        .words
        .iter()
        .any(|word| word.text.contains("example.invalid")));
    assert_eq!(
        check(&workflow("curl https://example.invalid/a > a.tgz")).len(),
        1
    );

    let words = commands("FOO=1 /usr/bin/curl");
    assert_eq!(
        words::command_name(&words[0].words).as_deref(),
        Some("curl")
    );
    assert!(words::is_assignment("FOO=1"));
    assert!(!words::is_assignment("=1"));
    assert!(!words::is_assignment("1=x"));
    assert_eq!(words::basename("C:\\bin\\curl"), "curl");
    assert!(words::is_downloader("wget", &[]));
    assert!(!words::is_downloader("echo", &["curl".to_string()]));
    assert!(words::command_name(&[]).is_none());
}

#[test]
fn urls_hosts_and_skip_comments_name_the_decision() {
    assert!(http_url("not-a-url").is_none());
    assert!(http_url("https://").is_none());
    assert!(http_url("https://example.invalid/$FILE").is_none());
    assert_eq!(
        http_url("HTTP://Example.Invalid/a").unwrap().host,
        "example.invalid"
    );
    assert!(
        http_url("https://example.invalid/a.SHA256")
            .unwrap()
            .checksum
    );
    assert!(
        http_url("https://example.invalid/SHA512SUMS.txt")
            .unwrap()
            .checksum
    );
    assert!(
        !http_url("https://example.invalid/{name}.tgz")
            .unwrap()
            .checksum
    );
    assert_eq!(http_url("https://[::1]/a").unwrap().host, "::1");
    assert!(http_url("https://[::1").is_none());
    assert_eq!(
        http_url("https://user:pass@example.invalid:8443/a?x=1")
            .unwrap()
            .host,
        "example.invalid"
    );
    assert_eq!(
        http_url("https://example.invalid:abc/a").unwrap().host,
        "example.invalid:abc"
    );
    assert!(configured_host("example.invalid").is_some());
    assert!(configured_host("example.invalid:443").is_some());
    assert!(configured_host("").is_none());
    assert!(configured_host("bad/host").is_none());
    assert!(configured_host("user@host").is_none());
    assert!(!url::trusted("example.invalid", &["other".to_string()]));

    assert!(judge::has_skip("# integrity-check: skip reason=vendor_1"));
    assert!(!judge::has_skip("# integrity-check: skip reason="));
    assert!(!judge::has_skip("# integrity-check: skip reason=has space"));
    assert!(!judge::has_skip(
        "# integrity-check: skipping reason=vendor"
    ));
    assert!(!judge::has_skip(
        "echo '# integrity-check: skip reason=vendor'"
    ));
    assert!(!judge::has_skip("echo no-hash"));

    let hits = download_hits(
        "curl --url=https://example.invalid/a -o out -- --url=https://example.invalid/b\nwget -O out https://example.invalid/SHA256SUMS\n",
        &[],
    );
    assert!(hits.iter().any(|hit| hit.url.contains("/a")));
    assert!(hits.iter().any(|hit| hit.command == "curl"));
    let trusted = judge::filter_trusted(hits, &["example.invalid".to_string()]);
    assert!(trusted.is_empty());
}

#[test]
fn verifiers_and_option_errors_are_explicit() {
    assert!(step_verifies("sha256sum --check sums"));
    assert!(step_verifies("sha256sum -cw sums"));
    assert!(step_verifies("sha256sum --check=sums"));
    assert!(step_verifies("shasum -a 256 -c sums"));
    assert!(step_verifies("shasum -a256 --check sums"));
    assert!(!step_verifies("sha256sum sums"));
    assert!(!step_verifies("shasum -a 512 -c sums"));
    assert!(!step_verifies("$(sha256sum) -c sums"));
    assert!(takes_url_slot("curl", "-o"));
    assert!(takes_url_slot("wget", "--directory-prefix"));
    assert!(!takes_url_slot("curl", "--url=https://example.invalid"));
    assert!(!takes_url_slot("curl", "--compressed"));
    assert!(!takes_url_slot("echo", "-o"));

    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("trustedHosts: ['bad/host']").unwrap(),
        ..Default::default()
    });
    let error = super::check_with_files(Path::new("."), &config, &[]).unwrap_err();
    assert!(error.to_string().contains("is not a host"), "{error}");
    config.rules[0].options = serde_yaml::from_str("downloadHelpers: ['1bad']").unwrap();
    let error = super::check_with_files(Path::new("."), &config, &[]).unwrap_err();
    assert!(
        error.to_string().contains("is not a command name"),
        "{error}"
    );
}
