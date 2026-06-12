use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn config_with_rule(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn fixture_root(subpath: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-test-correspondence/fixture")
            .join(subpath),
    )
}

#[test]
fn pass_fixture_has_no_findings() {
    let root = fixture_root("pass");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        findings.is_empty(),
        "expected no findings, got: {findings:?}"
    );
}

#[test]
fn fail_fixture_has_findings() {
    let root = fixture_root("fail");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        !findings.is_empty(),
        "expected findings for orphan test file"
    );
}

#[test]
fn test_to_source_direction_fixture_ignores_untested_sources() {
    let root = fixture_root("test-to-source-pass");
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let files = vec![
        root.join("backend/modules/format/index.mts"),
        root.join("backend/modules/format/index.test.mts"),
        root.join("backend/modules/untested/index.mts"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "test-to-source mode should ignore source files without tests: {findings:?}"
    );
}

#[test]
fn test_to_source_direction_fixture_reports_orphan_tests() {
    let root = fixture_root("test-to-source-fail");
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let files = vec![root.join("backend/modules/orphan/index.test.mts")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "expected one finding, got: {findings:?}");
    assert!(
        findings[0].message.contains("no corresponding source file"),
        "{}",
        findings[0].message
    );
}

#[test]
fn source_to_test_direction_fixture_ignores_orphan_tests() {
    let root = fixture_root("source-to-test-pass");
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let files = vec![root.join("backend/modules/orphan/index.test.mts")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "source-to-test mode should ignore tests without sources: {findings:?}"
    );
}

#[test]
fn source_to_test_direction_fixture_reports_untested_sources() {
    let root = fixture_root("source-to-test-fail");
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let files = vec![root.join("backend/modules/untested/index.mts")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "expected one finding, got: {findings:?}");
    assert!(
        findings[0].message.contains("no corresponding test file"),
        "{}",
        findings[0].message
    );
}

#[test]
fn test_to_source_direction_ignores_duplicate_test_stems() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend/mod")).unwrap();
    std::fs::write(root.join("backend/mod/index.ts"), "export {};\n").unwrap();
    std::fs::write(
        root.join("backend/mod/index.test.ts"),
        "test('a', () => {});\n",
    )
    .unwrap();
    std::fs::write(
        root.join("backend/mod/index.test.tsx"),
        "test('b', () => {});\n",
    )
    .unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.ts\", \".test.tsx\"], testsDir: __tests__, direction: test-to-source}",
    );
    let findings = check(root, &config).unwrap();
    assert!(
        findings.is_empty(),
        "test-to-source mode reports orphan tests only: {findings:?}"
    );
}

#[test]
fn tsx_source_with_ts_test_extension_fixture_has_findings() {
    let root = fixture_root("tsx-source-missing-test");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "backend/components/widget.tsx");
    assert!(findings[0].message.contains("no corresponding test file"));
}

#[test]
fn declaration_files_fixture_has_no_findings() {
    let root = fixture_root("declarations-pass");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn test_file_with_corresponding_source_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend/mod")).unwrap();
    std::fs::write(root.join("backend/mod/index.mts"), "export {};\n").unwrap();
    std::fs::write(
        root.join("backend/mod/index.test.mts"),
        "test('x', () => {});\n",
    )
    .unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.mts\"], testsDir: __tests__}",
    );
    let findings = check(root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn test_file_without_source_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend/mod")).unwrap();
    std::fs::write(
        root.join("backend/mod/index.test.mts"),
        "test('x', () => {});\n",
    )
    .unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.mts\"], testsDir: __tests__}",
    );
    let findings = check(root, &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("no corresponding source file"));
}

#[test]
fn test_file_in_tests_dir_is_exempt() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend/__tests__")).unwrap();
    std::fs::write(
        root.join("backend/__tests__/index.test.mts"),
        "test('x', () => {});\n",
    )
    .unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.mts\"], testsDir: __tests__}",
    );
    let findings = check(root, &config).unwrap();
    assert!(findings.is_empty(), "tests dir files should be exempt");
}

#[test]
fn out_of_scope_test_file_is_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("frontend/mod")).unwrap();
    std::fs::write(
        root.join("frontend/mod/index.test.mts"),
        "test('x', () => {});\n",
    )
    .unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.mts\"], testsDir: __tests__}",
    );
    let findings = check(root, &config).unwrap();
    assert!(findings.is_empty(), "out-of-scope files should be ignored");
}

#[test]
fn check_with_files_works() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend/mod")).unwrap();
    let test_file = root.join("backend/mod/index.test.mts");
    std::fs::write(&test_file, "test('x', () => {});\n").unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.mts\"], testsDir: __tests__}",
    );
    let findings = check_with_files(root, &config, &[test_file]).unwrap();
    assert_eq!(findings.len(), 1);
}

#[test]
fn ts_tsx_test_file_candidates() {
    let candidates = source_candidates("src/mod", "widget", ".test.ts");
    assert!(candidates.contains(&"src/mod/widget.ts".to_string()));
    assert!(candidates.contains(&"src/mod/widget.tsx".to_string()));
    assert!(candidates.contains(&"src/mod/index.ts".to_string()));
    assert!(candidates.contains(&"src/mod/index.tsx".to_string()));
}

#[test]
fn duplicate_group_can_collapse_first_dot_segment() {
    let root = fixture_root("duplicate-first-dot");
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.mts\"], testsDir: __tests__, duplicateStemGroup: first-dot-segment}",
    );
    let files = vec![
        root.join("backend/modules/report/index.mts"),
        root.join("backend/modules/report/index.test.mts"),
        root.join("backend/modules/report/index.edge.test.mts"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(
        findings.len(),
        2,
        "expected duplicate findings: {findings:?}"
    );
    assert!(findings
        .iter()
        .all(|finding| finding.message.contains("duplicate-stem")));
}

#[test]
fn source_candidates_empty_dir_has_no_slash_prefix() {
    // Exercises line 84: dir.is_empty() → String::new() (no path prefix).
    let candidates = source_candidates("", "widget", ".test.ts");
    assert!(candidates.contains(&"widget.ts".to_string()));
    assert!(candidates.contains(&"widget.tsx".to_string()));
    assert!(candidates.contains(&"index.ts".to_string()));
    assert!(candidates.contains(&"index.tsx".to_string()));
    // None should start with "/"
    assert!(candidates.iter().all(|c| !c.starts_with('/')));
}

#[test]
fn stem_and_dir_no_slash_returns_empty_dir() {
    // Exercises line 78: stem has no '/' → dir is empty string, base is stem.
    // This happens for a root-level test file like "index.test.mts".
    let (dir, base) = stem_and_dir("index.test.mts", ".test.mts");
    assert_eq!(dir, "");
    assert_eq!(base, "index");
}

#[test]
fn default_test_extensions_and_tests_dir_used_when_empty() {
    // Exercises lines 60 and 68: empty testExtensions / testsDir → defaults.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend/mod")).unwrap();
    std::fs::write(root.join("backend/mod/index.ts"), "export {};\n").unwrap();
    std::fs::write(
        root.join("backend/mod/index.test.ts"),
        "test('x', () => {});\n",
    )
    .unwrap();
    // Empty testExtensions and testsDir → use defaults
    let config = config_with_rule("{scopes: [backend]}");
    let findings = check(root, &config).unwrap();
    assert!(
        findings.is_empty(),
        "test with matching source should pass with default extensions"
    );
}

#[test]
fn duplicate_stem_test_files_are_flagged() {
    // Exercises lines 151-168: duplicate stem detection.
    // Two test files with the same stem in the same dir (not in __tests__)
    // should both be flagged.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend/mod")).unwrap();
    // Source file
    std::fs::write(root.join("backend/mod/index.ts"), "export {};\n").unwrap();
    // Two test files with stem "index" in the same directory
    std::fs::write(
        root.join("backend/mod/index.test.ts"),
        "test('a', () => {});\n",
    )
    .unwrap();
    std::fs::write(
        root.join("backend/mod/index.test.tsx"),
        "test('b', () => {});\n",
    )
    .unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.ts\", \".test.tsx\"], testsDir: __tests__}",
    );
    let findings = check(root, &config).unwrap();
    // Both test files should be flagged for duplicate stems
    let dup_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.message.contains("duplicate-stem"))
        .collect();
    assert!(
        !dup_findings.is_empty(),
        "duplicate stem test files should be flagged: {findings:?}"
    );
}

#[test]
fn source_candidates_js_extension() {
    // Exercises the "js" | "jsx" branch in source_candidates (lines 95-98).
    let candidates = source_candidates("src", "utils", ".test.js");
    assert!(candidates.contains(&"src/utils.js".to_string()));
    assert!(candidates.contains(&"src/utils.jsx".to_string()));
    assert!(candidates.contains(&"src/index.js".to_string()));
    assert!(candidates.contains(&"src/index.jsx".to_string()));
}

#[test]
fn source_candidates_jsx_extension() {
    let candidates = source_candidates("", "comp", ".test.jsx");
    assert!(candidates.contains(&"comp.js".to_string()));
    assert!(candidates.contains(&"comp.jsx".to_string()));
}

#[test]
fn source_candidates_mjs_extension() {
    // .test.mjs → only looks for .mjs sources, not .ts/.tsx
    let candidates = source_candidates("src", "utils", ".test.mjs");
    assert!(candidates.contains(&"src/utils.mjs".to_string()));
    assert!(candidates.contains(&"src/index.mjs".to_string()));
    assert!(!candidates.iter().any(|c| c.ends_with(".ts")));
    assert!(!candidates.iter().any(|c| c.ends_with(".tsx")));
}

#[test]
fn source_candidates_cjs_extension() {
    // .test.cjs → only looks for .cjs sources
    let candidates = source_candidates("src", "utils", ".test.cjs");
    assert!(candidates.contains(&"src/utils.cjs".to_string()));
    assert!(candidates.contains(&"src/index.cjs".to_string()));
    assert!(!candidates.iter().any(|c| c.ends_with(".ts")));
}

#[test]
fn source_file_without_test_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend/mod")).unwrap();
    std::fs::write(root.join("backend/mod/index.mts"), "export {};\n").unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.mts\"], testsDir: __tests__}",
    );
    let findings = check(root, &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("no corresponding test file"));
}

#[test]
fn non_source_extension_file_ignored_in_inverse_check() {
    // Exercises helpers.rs line 78: `return None` when file extension is not in
    // src_exts derived from test extensions.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend/mod")).unwrap();
    // .json file — not a source extension for .test.mts tests (src_ext = "mts")
    std::fs::write(root.join("backend/mod/config.json"), "{}").unwrap();
    // valid .mts source with its test
    std::fs::write(root.join("backend/mod/index.mts"), "export {};\n").unwrap();
    std::fs::write(
        root.join("backend/mod/index.test.mts"),
        "test('x', () => {});\n",
    )
    .unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.mts\"], testsDir: __tests__}",
    );
    let files = vec![
        root.join("backend/mod/config.json"),
        root.join("backend/mod/index.mts"),
        root.join("backend/mod/index.test.mts"),
    ];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "json file should be ignored in inverse check; everything else passes"
    );
}

#[test]
fn root_level_source_file_without_test_fails() {
    // Exercises helpers.rs lines 84 and 89: dir.is_empty() branches in
    // check_source_to_test when a source file is at the root level (no sub-directory).
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // Root-level source file — relative path has no '/', so dir="" in stem_and_dir.
    std::fs::write(root.join("root-module.ts"), "export {};\n").unwrap();
    let config = config_with_rule("{testExtensions: [\".test.ts\"], testsDir: __tests__}");
    let files = vec![root.join("root-module.ts")];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(
        findings[0].message.contains("no corresponding test file"),
        "root-level source without test should fail: {}",
        findings[0].message
    );
}

#[test]
fn stem_suffix_strip_pass_fixture_has_no_findings() {
    let root = fixture_root("stem-suffix-pass");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        findings.is_empty(),
        "mock.test file with stripped suffix should pass: {findings:?}"
    );
}

#[test]
fn stem_suffix_strip_test_to_source_resolves_mock() {
    let root = fixture_root("stem-suffix-mock-test-to-source");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        findings.is_empty(),
        "page.mock.test.tsx should resolve to page.tsx: {findings:?}"
    );
}

#[test]
fn stem_suffix_strip_source_to_test_finds_suffixed_test() {
    let root = fixture_root("stem-suffix-mock-source-to-test");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        findings.is_empty(),
        "page.tsx should find page.mock.test.tsx: {findings:?}"
    );
}

#[test]
fn stem_suffix_not_matching_base_does_not_crash() {
    let root = fixture_root("stem-suffix-no-match");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        findings.is_empty(),
        "non-matching suffix should not affect normal case: {findings:?}"
    );
}

#[test]
fn stem_suffix_source_to_test_in_tests_dir() {
    let root = fixture_root("stem-suffix-tests-dir");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        findings.is_empty(),
        "page.tsx should find __tests__/page.mock.test.tsx: {findings:?}"
    );
}
