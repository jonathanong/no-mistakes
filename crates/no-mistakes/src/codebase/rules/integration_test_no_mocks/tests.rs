use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/integration-test-no-mocks/unit-fixture")
        .join(name)
}

fn findings(name: &str) -> Vec<RuleFinding> {
    let root = fixture(name);
    let file = root.join("example.test.mts");
    let opts = Options::default();
    let compiled = compile_options(&opts).unwrap();
    check_file(&root, &file, &compiled)
}

#[test]
fn rejects_default_mock_calls_and_modules() {
    let findings = findings("defaults");

    assert_eq!(findings.len(), 6, "{findings:#?}");
    assert!(findings
        .iter()
        .any(|finding| finding.import.as_deref() == Some("vi.mock")));
    assert!(findings
        .iter()
        .any(|finding| finding.import.as_deref() == Some("vi.fn")));
    assert!(findings
        .iter()
        .any(|finding| finding.import.as_deref() == Some("msw")));
    assert!(findings
        .iter()
        .any(|finding| finding.import.as_deref() == Some("nock")));
    assert!(findings
        .iter()
        .any(|finding| finding.import.as_deref() == Some("sinon")));
}

#[test]
fn ignores_comments_and_global_fetch_router() {
    assert!(findings("comments").is_empty());
}

#[test]
fn strips_comments_and_strings_without_hiding_real_code() {
    let findings = findings("strings");

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_eq!(findings[0].line, 6);
    assert_eq!(findings[0].import.as_deref(), Some("vi.mock"));
    assert_eq!(findings[1].line, 7);
    assert_eq!(findings[1].import.as_deref(), Some("msw"));
}

#[test]
fn detects_modules_after_comment_markers_inside_strings() {
    let findings = findings("string-comment-marker");

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 3);
    assert_eq!(findings[0].import.as_deref(), Some("nock"));
}

#[test]
fn detects_wrapped_dynamic_imports_and_requires() {
    let findings = findings("wrapped");

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].import.as_deref(), Some("msw"));
    assert_eq!(findings[1].line, 4);
    assert_eq!(findings[1].import.as_deref(), Some("nock"));
}

#[test]
fn detects_calls_and_modules_inside_template_expressions() {
    let findings = findings("template-expression");

    assert_eq!(findings.len(), 4, "{findings:#?}");
    assert!(findings
        .iter()
        .any(|finding| finding.line == 1 && finding.import.as_deref() == Some("vi.mock")));
    assert!(findings
        .iter()
        .any(|finding| finding.line == 2 && finding.import.as_deref() == Some("msw")));
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.line == 3 && finding.import.as_deref() == Some("msw"))
            .count(),
        1
    );
    assert!(findings
        .iter()
        .any(|finding| finding.line == 3 && finding.import.as_deref() == Some("nock")));
}

#[test]
fn strip_helpers_preserve_offsets_for_unclosed_and_escaped_tokens() {
    assert_eq!(
        strip::comments("before /* unclosed\nvi.fn()"),
        "before            \n       "
    );
    assert_eq!(
        strip::comments_and_strings("const value = 'escaped\\' quote'\nconst tail = `open"),
        "const value =                  \nconst tail =      "
    );
    assert_eq!(
        strip::comments_and_strings("const tail = 'open\\"),
        "const tail =       "
    );
    let complex_template =
        "const value = `${/* hidden */ ({ nested: `text ${vi.fn()}` }) // tail\n}`";
    let stripped = strip::comments_and_strings(complex_template);
    assert!(stripped.contains("vi.fn()"), "{stripped}");
    assert!(!stripped.contains("hidden"), "{stripped}");
    assert!(!stripped.contains("text"), "{stripped}");
    assert_eq!(
        strip::comments_and_strings("const value = `open\\"),
        "const value =       "
    );
    assert_eq!(
        strip::comments_and_strings("const value = `open\\x`"),
        "const value =         "
    );
    assert_eq!(
        strip::comments_and_strings("const value = `${unterminated"),
        "const value =    unterminated"
    );
    assert_eq!(
        strip::comments("const value = `open\\"),
        "const value = `open\\"
    );
}

#[test]
fn module_matches_ignore_closed_string_literals_before_real_imports() {
    let results = findings("string-before-real");

    assert_eq!(results.len(), 1, "{results:#?}");
    assert_eq!(results[0].import.as_deref(), Some("msw"));
    assert!(findings("string-only").is_empty());
    assert!(findings("escaped-string-only").is_empty());

    let nested = b"`${condition ? `${value}` : { value: require('nock') }}` after";
    let after = nested
        .windows("after".len())
        .position(|window| window == b"after")
        .unwrap();
    assert!(!strings::is_inside_string(nested, after));
}

#[test]
fn custom_call_and_module_options_replace_defaults() {
    let opts = Options {
        forbidden_calls: vec!["mockLib.fake".to_string()],
        forbidden_modules: vec!["wiremock".to_string()],
    };
    let compiled = compile_options(&opts).unwrap();
    let root = fixture("custom");
    let file = root.join("case.mts");

    let findings = check_file(&root, &file, &compiled);

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings
        .iter()
        .any(|finding| finding.import.as_deref() == Some("mockLib.fake")));
    assert!(findings
        .iter()
        .any(|finding| finding.import.as_deref() == Some("wiremock")));
}

#[test]
fn extensionless_custom_call_and_missing_file_paths_are_handled() {
    let opts = Options {
        forbidden_calls: vec!["mock".to_string()],
        forbidden_modules: Vec::new(),
    };
    let compiled = compile_options(&opts).unwrap();
    let root = fixture("extensionless");
    let file = root.join("case.mts");

    let findings = check_file(&root, &file, &compiled);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].import.as_deref(), Some("mock"));

    let missing = root.join("missing.mts");
    assert!(check_file(&root, &missing, &compiled).is_empty());
}
