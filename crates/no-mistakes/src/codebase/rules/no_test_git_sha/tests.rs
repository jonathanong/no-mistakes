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

// Output cases moved to cli_rra_snapshot_no_test_git_sha. Keep nested public
// discovery, missing sources, config errors, and prefilter counters.

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

fn allow_audit_config() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("allowedContexts: ['intentional', 'intent.*', 'clean']")
                .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn allow_audit_tracks_raw_findings_and_all_overlapping_entries() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/no-test-git-sha/fixture/allow-audit");
    let files = vec![root.join("dirty.txt"), root.join("clean.txt")];
    let sources = super::super::source_store_for_files(&files);
    let findings =
        check_with_files_and_sources(&root, &allow_audit_config(), &files, &sources).unwrap();
    let stale: Vec<_> = findings
        .iter()
        .filter(|finding| finding.rule == "unused-allow-entry")
        .collect();
    assert_eq!(stale.len(), 1, "{findings:?}");
    assert_eq!(stale[0].target.as_deref(), Some("clean"));
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn allow_audit_requires_the_full_application_universe() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/no-test-git-sha/fixture/allow-audit");
    let inventory = vec![root.join("dirty.txt"), root.join("clean.txt")];
    let sources = super::super::source_store_for_files(&inventory);
    let findings =
        check_with_files_and_sources(&root, &allow_audit_config(), &inventory[1..], &sources)
            .unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| finding.rule != "unused-allow-entry"),
        "{findings:?}"
    );
}

#[test]
fn allow_audit_is_silent_when_relevant_input_is_unreadable() {
    let root = fixture("allow-audit");
    let files = vec![root.join("missing.txt")];
    let sources = super::super::source_store_for_files(&files);
    let findings =
        check_with_files_and_sources(&root, &allow_audit_config(), &files, &sources).unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| finding.rule != "unused-allow-entry"),
        "{findings:?}"
    );
}
