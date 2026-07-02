use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleTestTargets},
    NoMistakesConfig,
};
use std::path::PathBuf;

mod strip_helpers;

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

    assert_eq!(findings.len(), 8, "{findings:#?}");
    assert!(findings
        .iter()
        .any(|finding| finding.import.as_deref() == Some("vi.mock")));
    assert!(findings
        .iter()
        .any(|finding| finding.import.as_deref() == Some("vi.fn")));
    assert!(findings
        .iter()
        .any(|finding| finding.import.as_deref() == Some("jest.fn")));
    assert!(findings
        .iter()
        .any(|finding| finding.import.as_deref() == Some("jest.spyOn")));
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
fn detects_wrapped_forbidden_calls_across_lines() {
    let findings = findings("wrapped-calls");

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings
        .iter()
        .any(|finding| finding.line == 1 && finding.import.as_deref() == Some("vi.mock")));
    assert!(findings
        .iter()
        .any(|finding| finding.line == 4 && finding.import.as_deref() == Some("vi.mock")));
}

#[test]
fn ignores_comments_and_global_fetch_router() {
    assert!(findings("comments").is_empty());
}

#[test]
fn test_target_only_rules_keep_files_for_include_filtering() {
    let root = fixture("defaults");
    let file = root.join("example.test.mts");
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            tests: RuleTestTargets {
                vitest: vec!["integration".to_string()],
                ..Default::default()
            },
            include: vec!["**/*.test.mts".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let findings = check_with_files(&root, &config, &[file]).unwrap();

    assert!(!findings.is_empty(), "expected included test file findings");
}

#[test]
fn test_target_rules_with_include_stay_inside_selected_test_project() {
    let root = fixture("defaults");
    let unit = root.join("example.test.mts");
    let integration = root.join("integration-tests/example.test.mts");
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            tests: RuleTestTargets {
                vitest: vec!["integration".to_string()],
                ..Default::default()
            },
            include: vec!["**/*.test.mts".to_string()],
            ..Default::default()
        }],
        tests: crate::config::v2::schema::Tests {
            vitest: crate::config::v2::schema::VitestConfig {
                projects: std::collections::BTreeMap::from([(
                    "integration".to_string(),
                    crate::config::v2::schema::TestProjectPolicy {
                        include: vec!["integration-tests/**/*.test.mts".to_string()],
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    let skip = super::super::skip_dir_set(&config);

    let files = candidate_files(
        &root,
        &config,
        &[unit, integration.clone()],
        &skip,
        &[],
        &config.rules[0],
    );

    assert_eq!(files, vec![integration]);
}

#[test]
fn test_target_only_rules_use_configured_test_file_filter() {
    let root = fixture("defaults");
    let file = root.join("example.test.mts");
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            tests: RuleTestTargets {
                vitest: vec!["integration".to_string()],
                ..Default::default()
            },
            ..Default::default()
        }],
        tests: crate::config::v2::schema::Tests {
            vitest: crate::config::v2::schema::VitestConfig {
                projects: std::collections::BTreeMap::from([(
                    "integration".to_string(),
                    crate::config::v2::schema::TestProjectPolicy {
                        include: vec!["integration-tests/**/*.test.mts".to_string()],
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let findings = check_with_files(&root, &config, &[file]).unwrap();

    assert!(
        findings.is_empty(),
        "unmatched test file should not be scanned: {findings:#?}"
    );
}

#[test]
fn selected_test_target_match_handles_playwright_excludes_and_empty_policies() {
    let root = fixture("defaults");
    let config = NoMistakesConfig {
        tests: crate::config::v2::schema::Tests {
            playwright: crate::config::v2::schema::PlaywrightTestConfig {
                projects: std::collections::BTreeMap::from([
                    (
                        "e2e".to_string(),
                        crate::config::v2::schema::TestProjectPolicy {
                            include: vec!["tests/e2e/**/*.test.mts".to_string()],
                            exclude: vec!["tests/e2e/skip/**/*.test.mts".to_string()],
                            ..Default::default()
                        },
                    ),
                    (
                        "empty".to_string(),
                        crate::config::v2::schema::TestProjectPolicy::default(),
                    ),
                ]),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    let rule = RuleDef {
        rule: RULE_ID.to_string(),
        tests: RuleTestTargets {
            playwright: vec!["e2e".to_string(), "empty".to_string()],
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(test_targets::selected_match(
        &root,
        &config,
        &rule,
        &root.join("tests/e2e/login.test.mts")
    ));
    assert!(!test_targets::selected_match(
        &root,
        &config,
        &rule,
        &root.join("tests/e2e/skip/login.test.mts")
    ));
}

#[test]
fn selected_test_target_match_falls_back_when_named_policy_has_no_include() {
    let root = fixture("defaults");
    let config = NoMistakesConfig {
        tests: crate::config::v2::schema::Tests {
            vitest: crate::config::v2::schema::VitestConfig {
                projects: std::collections::BTreeMap::from([(
                    "integration".to_string(),
                    crate::config::v2::schema::TestProjectPolicy {
                        integration_suites: std::collections::BTreeMap::from([(
                            "openai".to_string(),
                            vec!["openai".to_string()],
                        )]),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    let rule = RuleDef {
        rule: RULE_ID.to_string(),
        tests: RuleTestTargets {
            vitest: vec!["integration".to_string()],
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(test_targets::selected_match(
        &root,
        &config,
        &rule,
        &root.join("src/example.test.mts")
    ));
    assert!(!test_targets::selected_match(
        &root,
        &config,
        &rule,
        &root.join("src/example.mts")
    ));
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
fn detects_bracket_and_typed_forbidden_calls() {
    let findings = findings("bracket-typed-calls");

    assert_eq!(findings.len(), 5, "{findings:#?}");
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.line, finding.import.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            (1, Some("vi.mock")),
            (2, Some("vi.fn")),
            (4, Some("vi.fn")),
            (5, Some("jest.fn")),
            (3, Some("jest.spyOn"))
        ]
    );
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
fn ignores_module_specifiers_inside_regex_literals() {
    let findings = findings("regex-literal");

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 3);
    assert_eq!(findings[0].import.as_deref(), Some("sinon"));
}

#[test]
fn detects_calls_and_modules_inside_template_expressions() {
    let findings = findings("template-expression");

    assert_eq!(findings.len(), 6, "{findings:#?}");
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
    assert!(findings
        .iter()
        .any(|finding| finding.line == 4 && finding.import.as_deref() == Some("msw")));
    assert!(findings
        .iter()
        .any(|finding| finding.line == 5 && finding.import.as_deref() == Some("nock")));
    assert!(findings.iter().all(|finding| finding.line != 6
        && finding.line != 7
        && finding.line != 8
        && finding.line != 9));
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

    let block_comment = b"`${/* await import('msw/node') */ value}`";
    let commented_import = block_comment
        .windows("import".len())
        .position(|window| window == b"import")
        .unwrap();
    assert!(strings::is_inside_string(block_comment, commented_import));

    let line_comment = b"`${// require('nock')\nvalue}`";
    let commented_require = line_comment
        .windows("require".len())
        .position(|window| window == b"require")
        .unwrap();
    assert!(strings::is_inside_string(line_comment, commented_require));
    let after_line_comment = line_comment
        .windows("value".len())
        .position(|window| window == b"value")
        .unwrap();
    assert!(!strings::is_inside_string(line_comment, after_line_comment));

    let normal_line_comment = b"// require('nock')\nconst value = 1";
    let normal_line_comment_import = normal_line_comment
        .windows("require".len())
        .position(|window| window == b"require")
        .unwrap();
    assert!(strings::is_inside_string(
        normal_line_comment,
        normal_line_comment_import
    ));

    let normal_block_comment = b"/* require('nock') */ const value = 1";
    let normal_block_comment_import = normal_block_comment
        .windows("require".len())
        .position(|window| window == b"require")
        .unwrap();
    assert!(strings::is_inside_string(
        normal_block_comment,
        normal_block_comment_import
    ));
    let after_block_comment = normal_block_comment
        .windows("const".len())
        .position(|window| window == b"const")
        .unwrap();
    assert!(!strings::is_inside_string(
        normal_block_comment,
        after_block_comment
    ));
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
