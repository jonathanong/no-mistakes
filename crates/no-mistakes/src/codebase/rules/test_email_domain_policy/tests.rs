use super::*;

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

fn findings(path: &str, source: &str) -> Vec<RuleFinding> {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join(path);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, source).unwrap();
    check_file(tmp.path(), &file, &compiled())
}

#[test]
fn rejects_raw_and_encoded_banned_domains() {
    let findings = findings(
        "src/send.test.mts",
        "const a = 'person@example.com'\nconst b = 'person%40example%2Ecom'\nconst c = 'person%40tag%40example.com%26otp'\n",
    );

    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some("example.com")));
    assert!(findings[0].message.contains("tests+<hash>@voucha.ai"));
}

#[test]
fn allows_configured_pattern_urls_and_longer_domains() {
    let findings = findings(
        "src/send.test.mts",
        "const ok = ['tests+person@voucha.ai', 'tests%2Bperson%40voucha.ai', 'https://example.com/path', 'user@example.company']\n",
    );

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn skips_unconfigured_extensions() {
    let findings = findings("images/photo.png", "person@example.com\n");
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn empty_banned_domains_is_noop() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("src/send.test.mts");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "person@example.com\n").unwrap();
    let findings = scan(tmp.path(), &Options::default(), &[file]).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn covers_custom_extensions_no_replacement_invalid_regex_and_missing_files() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("fixtures/email.fixture");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "person@example.com\n").unwrap();

    let opts = Options {
        banned_domains: vec!["example.com.".to_string()],
        allowed_email_patterns: Vec::new(),
        replacement: None,
        extensions: vec![".fixture".to_string()],
    };
    let findings = scan(
        tmp.path(),
        &opts,
        &[file.clone(), tmp.path().join("fixtures/missing.fixture")],
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
