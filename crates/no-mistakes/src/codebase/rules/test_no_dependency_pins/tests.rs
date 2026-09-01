use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules/test-no-dependency-pins")
}

fn fixture(scenario: &str) -> PathBuf {
    fixture_root().join("fixture").join(scenario)
}

fn unit_fixture(name: &str) -> PathBuf {
    fixture_root().join("unit-fixture").join(name)
}

fn config_with_options(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn fail_files(root: &Path) -> Vec<PathBuf> {
    vec![
        root.join("src/action-ref.test.mts"),
        root.join("src/tool-version.test.mts"),
        root.join("src/release-url.test.mts"),
        root.join("src/release-asset.test.mts"),
        root.join("src/tool-log.test.mts"),
        root.join("src/dependency-manifest.test.mts"),
        root.join("src/__tests__/nested.ts"),
        root.join("src/helper.mock.test.js"),
    ]
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let sources = super::super::source_store_for_files(files);
    scan_with_sources(root, opts, files, &sources)
}

#[test]
fn default_include_matches_filaments_test_file_re() {
    let re = default_include_regex();
    for path in [
        "foo.test.ts",
        "src/foo.test.mts",
        "src/foo.mock.test.ts",
        "src/foo.test.tsx",
        "src/foo.test.mjs",
        "src/foo.test.js",
        "src/foo.test.cts",
        "src/foo.test.cjs",
        "__tests__/helper.ts",
        "src/__tests__/nested/helper.tsx",
    ] {
        assert!(re.is_match(path), "{path}");
    }
    for path in [
        "src/foo.ts",
        "src/foo.spec.ts",
        "src/foo.test.rs",
        "tests/foo.ts",
        "src/foo.test",
    ] {
        assert!(!re.is_match(path), "{path}");
    }
}

#[test]
fn fail_fixture_reports_all_pin_shapes() {
    let root = fixture("fail");
    let findings = check_with_files(&root, &config_with_options("{}"), &fail_files(&root)).unwrap();
    let reasons: Vec<&str> = findings
        .iter()
        .filter_map(|finding| finding.target.as_deref())
        .collect();

    assert!(
        findings
            .iter()
            .all(|finding| finding.rule == RULE_ID && finding.line > 0),
        "{findings:#?}"
    );
    for reason in [
        "exact action ref",
        "exact tool version",
        "versioned release URL",
        "versioned release asset",
        "versioned tool log",
        "package.json dependency assertion",
        "parsed dependency version assertion",
    ] {
        assert!(reasons.contains(&reason), "missing {reason}: {findings:#?}");
    }
    assert!(findings.iter().any(|finding| {
        finding.file == "src/action-ref.test.mts"
            && finding.import.as_deref() == Some("actions/checkout@v6.0.2")
    }));
    assert!(findings.iter().any(|finding| {
        finding.import.as_deref()
            == Some("actions/setup-node@de0fac2e4500dabe0009e67214ff5f5447ce83dd")
    }));
    assert!(findings.iter().any(|finding| {
        finding.file == "src/dependency-manifest.test.mts"
            && finding.target.as_deref() == Some("package.json dependency assertion")
    }));
    assert_eq!(
        findings
            .iter()
            .filter(|finding| {
                finding.file == "src/dependency-manifest.test.mts"
                    && finding.target.as_deref() == Some("package.json dependency assertion")
            })
            .count(),
        3,
        "{findings:#?}"
    );
    assert_eq!(
        findings
            .iter()
            .filter(|finding| {
                finding.file == "src/dependency-manifest.test.mts"
                    && finding.target.as_deref() == Some("parsed dependency version assertion")
            })
            .count(),
        10,
        "{findings:#?}"
    );
    let multiline = findings
        .iter()
        .find(|finding| {
            finding
                .import
                .as_deref()
                .is_some_and(|pin| pin.contains("@typescript-eslint/parser"))
        })
        .expect("multiline dependency assertion finding");
    assert_eq!(multiline.line, 13, "{multiline:#?}");
    assert!(!multiline.message.contains('\n'), "{multiline:#?}");
    assert!(findings
        .iter()
        .any(|finding| finding.file == "src/__tests__/nested.ts"));
    assert!(findings
        .iter()
        .any(|finding| finding.file == "src/helper.mock.test.js"));
}

#[test]
fn negatives_and_non_test_files_are_ignored() {
    let root = fixture("pass");
    let files = vec![
        root.join("src/negatives.test.mts"),
        root.join("src/not-a-test.ts"),
        root.join("src/installer.spec.ts"),
    ];
    let findings = check_with_files(&root, &config_with_options("{}"), &files).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn honors_disable_comments() {
    let root = fixture("pass");
    let files = vec![
        root.join("src/line-disabled.test.mts"),
        root.join("src/file-disabled.test.mts"),
        root.join("src/next-line-disabled.test.mts"),
    ];
    let mut findings = check_with_files(&root, &config_with_options("{}"), &files).unwrap();
    assert_eq!(findings.len(), 3, "{findings:#?}");
    let sources = super::super::source_store_for_files(&files);
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn custom_include_globs_replace_default_test_file_re() {
    let root = fixture("fail");
    let files = fail_files(&root);
    let only_action = check_with_files(
        &root,
        &config_with_options("include: ['**/action-ref.test.mts']"),
        &files,
    )
    .unwrap();
    assert!(
        only_action
            .iter()
            .all(|finding| finding.file == "src/action-ref.test.mts"),
        "{only_action:#?}"
    );
    assert!(!only_action.is_empty(), "{only_action:#?}");

    let none = check_with_files(
        &root,
        &config_with_options("include: ['**/does-not-exist.test.ts']"),
        &files,
    )
    .unwrap();
    assert!(none.is_empty(), "{none:#?}");
}

#[test]
fn custom_patterns_replace_defaults() {
    let root = unit_fixture("custom");
    let file = root.join("src/pin.test.mts");
    let findings = scan(
        &root,
        &Options {
            include: Vec::new(),
            patterns: vec![PatternOption {
                reason: "custom pin".to_string(),
                regex: r"PINNED_TOOL:\s*\d+\.\d+\.\d+".to_string(),
            }],
        },
        &[file],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("custom pin"));
    assert_eq!(findings[0].import.as_deref(), Some("PINNED_TOOL: 9.9.9"));
    assert!(!findings[0].message.contains("exact action ref"));
}

#[test]
fn invalid_custom_regex_and_include_glob_error() {
    let invalid_regex = Options {
        include: Vec::new(),
        patterns: vec![PatternOption {
            reason: "bad".to_string(),
            regex: "[".to_string(),
        }],
    };
    let error = compile_options(&invalid_regex)
        .err()
        .expect("invalid regex")
        .to_string();
    assert!(error.contains(RULE_ID), "{error}");
    assert!(error.contains("invalid pattern"), "{error}");

    let invalid_glob = Options {
        include: vec!["[".to_string()],
        patterns: Vec::new(),
    };
    let error = compile_options(&invalid_glob)
        .err()
        .expect("invalid include glob")
        .to_string();
    assert!(error.contains("invalid glob"), "{error}");
}

#[test]
fn missing_file_and_action_ref_lookbehind() {
    let root = fixture("pass");
    let findings = scan(
        &root,
        &Options::default(),
        &[
            root.join("src/missing.test.mts"),
            root.join("src/negatives.test.mts"),
        ],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");

    let compiled = compile_options(&Options::default()).unwrap();
    let findings = check_source(
        "src/start.test.mts",
        "actions/checkout@v4\n@actions/checkout@v4\n",
        &compiled,
    );
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].import.as_deref(), Some("actions/checkout@v4"));
}

#[test]
fn check_entry_point_uses_discovery() {
    let root = fixture("pass");
    let findings = super::check(&root, &config_with_options("{}")).unwrap();
    assert!(
        findings.iter().all(|finding| finding.rule == RULE_ID),
        "{findings:#?}"
    );
}

#[test]
fn message_includes_file_line_reason_and_match() {
    assert_eq!(
        super::scan::message(
            "src/app.test.ts",
            4,
            "exact action ref",
            "actions/checkout@v4",
        ),
        "src/app.test.ts:4: tests must not pin exact dependency versions (exact action ref): `actions/checkout@v4`"
    );
}
