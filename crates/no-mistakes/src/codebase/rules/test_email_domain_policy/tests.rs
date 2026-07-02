use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/test-email-domain-policy/unit-fixture")
        .join(name)
}

fn compiled() -> CompiledOptions {
    compile_options(&Options {
        banned_domains: vec!["example.com".to_string()],
        allowed_email_patterns: vec![
            r"(?i)^tests(?:\+[a-z0-9._%${}-]+|%2b[a-z0-9._%${}-]+)(?:@|%40)voucha\.ai$".to_string(),
        ],
        replacement: Some("tests+<hash>@voucha.ai".to_string()),
        extensions: Vec::new(),
    })
    .unwrap()
}

fn findings(name: &str, path: &str) -> Vec<RuleFinding> {
    let root = fixture(name);
    let file = root.join(path);
    check_file(&root, &file, &compiled())
}

#[test]
fn rejects_raw_and_encoded_banned_domains() {
    let findings = findings("raw", "src/send.test.mts");

    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some("example.com")));
    assert!(findings[0].message.contains("tests+<hash>@voucha.ai"));
}

#[test]
fn allows_configured_pattern_urls_and_longer_domains() {
    let findings = findings("allowed", "src/send.test.mts");

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn skips_unconfigured_extensions() {
    let findings = findings("skipped", "images/photo.png");
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn empty_banned_domains_is_noop() {
    let root = fixture("empty");
    let file = root.join("src/send.test.mts");
    let findings = scan(&root, &Options::default(), &[file]).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn covers_custom_extensions_no_replacement_invalid_regex_and_missing_files() {
    let root = fixture("custom");
    let file = root.join("fixtures/email.fixture");

    let opts = Options {
        banned_domains: vec!["example.com.".to_string()],
        allowed_email_patterns: Vec::new(),
        replacement: None,
        extensions: vec![".fixture".to_string()],
    };
    let findings = scan(
        &root,
        &opts,
        &[file.clone(), root.join("fixtures/missing.fixture")],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(
        findings[0].message,
        "fixtures/email.fixture: test email fixtures must not use `example.com`"
    );

    let invalid = Options {
        banned_domains: vec!["example.com".to_string()],
        allowed_email_patterns: vec!["[".to_string()],
        replacement: None,
        extensions: Vec::new(),
    };
    assert!(compile_options(&invalid).is_err());

    assert_eq!(email_domain("not-an-email"), "");
    assert_eq!(email_domain("person%40EXAMPLE%2ECOM%26otp"), "example.com");
}
