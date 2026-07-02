use super::*;

fn findings(source: &str) -> Vec<RuleFinding> {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp
        .path()
        .join("integration-tests/web-api/example.test.mts");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, source).unwrap();
    let opts = Options::default();
    let compiled = compile_options(&opts).unwrap();
    check_file(tmp.path(), &file, &compiled)
}

#[test]
fn rejects_default_mock_calls_and_modules() {
    let source = "\
import { vi } from 'vitest'
vi.mock('../module')
const fn = vi.fn()
const server = await import('msw/node')
const nock = require('nock')
import sinon from 'sinon'
import 'msw'
";
    let findings = findings(source);

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
    let source = "\
// vi.mock('foo') is forbidden by policy
/* vi.fn() */
 * vi.spyOn(console, 'log')
globalThis.fetch = previousFetch
";
    assert!(findings(source).is_empty());
}

#[test]
fn strips_comments_and_strings_without_hiding_real_code() {
    let source = "\
const message = 'vi.mock should only be text'
const other = \"import('msw') is documentation\"
/*
vi.fn()
*/
/* setup */ vi.mock('../module')
const server = await import('msw/node')
";
    let findings = findings(source);

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_eq!(findings[0].line, 6);
    assert_eq!(findings[0].import.as_deref(), Some("vi.mock"));
    assert_eq!(findings[1].line, 7);
    assert_eq!(findings[1].import.as_deref(), Some("msw"));
}

#[test]
fn detects_modules_after_comment_markers_inside_strings() {
    let source = r#"
const url = "http://example.test"
const nock = require("nock")
"#;
    let findings = findings(source);

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 3);
    assert_eq!(findings[0].import.as_deref(), Some("nock"));
}

#[test]
fn detects_wrapped_dynamic_imports_and_requires() {
    let source = "\
const server = await import(
  'msw/node'
)
const nock = require(
  'nock'
)
";
    let findings = findings(source);

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].import.as_deref(), Some("msw"));
    assert_eq!(findings[1].line, 4);
    assert_eq!(findings[1].import.as_deref(), Some("nock"));
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
}

#[test]
fn module_matches_ignore_closed_string_literals_before_real_imports() {
    let results = findings(r#"const note = "import('msw')"; import real from 'msw/node'"#);

    assert_eq!(results.len(), 1, "{results:#?}");
    assert_eq!(results[0].import.as_deref(), Some("msw"));
    assert!(findings(r#"const note = "import('msw/node')""#).is_empty());
    assert!(findings(r#"const note = "escaped \\ before import('msw/node')""#).is_empty());
}

#[test]
fn custom_call_and_module_options_replace_defaults() {
    let opts = Options {
        forbidden_calls: vec!["mockLib.fake".to_string()],
        forbidden_modules: vec!["wiremock".to_string()],
    };
    let compiled = compile_options(&opts).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("case.mts");
    std::fs::write(
        &file,
        "vi.mock('allowed-by-custom-config')\nmockLib.fake()\nimport x from 'wiremock/node'\n",
    )
    .unwrap();

    let findings = check_file(tmp.path(), &file, &compiled);

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
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("case.mts");
    std::fs::write(&file, "mock()\n").unwrap();

    let findings = check_file(tmp.path(), &file, &compiled);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].import.as_deref(), Some("mock"));

    let missing = tmp.path().join("missing.mts");
    assert!(check_file(tmp.path(), &missing, &compiled).is_empty());
}
