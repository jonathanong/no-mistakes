use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/no-test-git-sha/fixture")
        .join(name)
}

fn review_fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/repository-guard-review")
        .join(name)
}

#[test]
fn reports_selected_full_sha_and_honors_allowed_context() {
    let root = fixture("fail");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = vec![root.join("tests/example.test.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 1);
}

#[test]
fn public_check_handles_a_nested_fixture_root() {
    let root = fixture("fail");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    // The fixture is inside this repository's Git worktree; public discovery
    // must still be safe when the requested root is a nested directory.
    assert!(check(&root, &config).unwrap().is_empty());
}

#[test]
fn unavailable_source_is_ignored() {
    let root = fixture("fail");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let missing = root.join("tests/missing.test.ts");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&missing));
    assert!(
        check_with_files_and_sources(&root, &config, &[missing], &sources)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn standard_include_selects_only_configured_paths() {
    let root = fixture("pass");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = vec![root.join("src/value.ts")];
    assert!(check_with_files(&root, &config, &files).unwrap().is_empty());
}

#[test]
fn selected_source_without_a_sha_is_clean() {
    let root = fixture("pass");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        include: vec!["tests/**/*.test.ts".to_string()],
        ..Default::default()
    });
    let file = root.join("tests/no-sha.test.ts");
    assert!(check_with_files(&root, &config, &[file])
        .unwrap()
        .is_empty());
}

#[test]
fn invalid_allowed_context_regex_is_a_configuration_error() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("allowedContexts: ['(']").unwrap(),
        ..Default::default()
    });
    let root = fixture("fail");
    let error =
        check_with_files(&root, &config, &[root.join("tests/example.test.ts")]).unwrap_err();
    assert!(error.to_string().contains("allowedContexts"), "{error:#}");
}

#[test]
fn standard_exclude_removes_a_selected_test_path() {
    let root = fixture("fail");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        include: vec!["tests/**/*.test.ts".to_string()],
        exclude: vec!["tests/example.test.ts".to_string()],
        ..Default::default()
    });
    assert!(
        check_with_files(&root, &config, &[root.join("tests/example.test.ts")])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn hexadecimal_boundaries_allow_underscores_but_reject_longer_hex_values() {
    let root = review_fixture("no-test-git-sha");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let file = root.join("tests/boundaries.test.ts");

    let findings = check_with_files(&root, &config, &[file]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 1, "{findings:?}");
}

#[test]
fn prefilter_preserves_fixture_findings() {
    let cases = [
        fixture("fail"),
        fixture("pass"),
        review_fixture("no-test-git-sha"),
    ];
    for root in cases {
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
        let files = crate::codebase::ts_source::discover_files(&root, &[]);
        let sources = super::super::source_store_for_files(&files);
        let rule = config
            .rule_applications(RULE_ID)
            .into_iter()
            .next()
            .expect("fixture enables no-test-git-sha");
        let options = compile_options(rule.try_rule_options().unwrap()).unwrap();
        for path in &files {
            let filtered = check_file(&root, path, &options, &sources);
            let Some(content) = super::super::read_source(&sources, path) else {
                assert!(filtered.is_empty(), "{}", path.display());
                continue;
            };
            let unfiltered = scan_sha_lines(&root, path, &content, &options);
            assert_eq!(filtered, unfiltered, "{}", path.display());
        }
    }
}

#[test]
fn prefilter_skips_files_without_hex_runs() {
    super::PREFILTER_SKIPS.with(|skips| skips.set(0));
    let root = fixture("pass");
    let path = root.join("src/plain.ts");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    let rule = config
        .rule_applications(RULE_ID)
        .into_iter()
        .next()
        .expect("fixture enables no-test-git-sha");
    let options = compile_options(rule.try_rule_options().unwrap()).unwrap();
    let findings = check_file(&root, &path, &options, &sources);
    assert!(findings.is_empty(), "{findings:?}");
    assert!(
        super::PREFILTER_SKIPS.with(|skips| skips.get()) >= 1,
        "files without a 40-digit hex run must skip the SHA regex"
    );
}
