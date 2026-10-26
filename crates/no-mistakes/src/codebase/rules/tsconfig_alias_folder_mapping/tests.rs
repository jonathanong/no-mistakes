use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/tsconfig-alias-folder-mapping/fixture")
        .join(path)
}

fn config_from_yaml(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn agents_config() -> &'static str {
    "tsconfig: tsconfig.json\nbaseDir: backend\nmappings:\n  - prefix: \"@agents\"\n    root: agents\n"
}

fn findings(name: &str) -> Vec<RuleFinding> {
    check(&fixture(name), &config_from_yaml(agents_config())).unwrap()
}

#[test]
fn directory_and_extension_qualified_mappings_pass() {
    for name in ["pass", "valid-files"] {
        assert!(findings(name).is_empty(), "{name}: {:?}", findings(name));
    }
}

#[test]
fn genuine_mismatches_are_still_reported_at_each_entry() {
    let findings = findings("wrong-files");
    assert_eq!(findings.len(), 9, "{findings:?}");
    assert!(findings.iter().all(|finding| finding.line > 1));
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.message.contains("must use prefix"))
            .count(),
        1
    );
}

#[test]
fn fail_fixture_produces_findings() {
    let findings = findings("fail");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("must target"));
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].file, "tsconfig.json");
}

#[test]
fn no_options_is_noop() {
    assert!(check(&fixture("fail"), &config_from_yaml("{}"))
        .unwrap()
        .is_empty());
}

#[test]
fn existing_unloadable_tsconfig_is_a_finding() {
    let findings = findings("extends-missing");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].file, "tsconfig.json");
    assert!(findings[0]
        .message
        .contains("tsconfig could not be loaded and alias mappings were not checked"));
}

#[test]
fn missing_tsconfig_skips_invalid_rule_path_filters() {
    let mut config =
        config_from_yaml(&agents_config().replace("tsconfig.json", "nonexistent.json"));
    // Preserve the no-op before path filter compilation, including its error precedence.
    config.rules[0].include = vec!["[".to_string()];
    assert!(check(&fixture("pass"), &config).unwrap().is_empty());
}

#[test]
fn check_with_files_delegates_to_tsconfig() {
    let findings =
        check_with_files(&fixture("fail"), &config_from_yaml(agents_config()), &[]).unwrap();
    assert_eq!(findings.len(), 1);
}

#[test]
fn multiple_targets_all_checked() {
    let findings = findings("multiple-targets");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("backend/modules/email"));
}

#[test]
fn no_findings_when_no_compiler_options_paths() {
    assert!(findings("no-paths").is_empty());
}

#[test]
fn wrong_prefix_for_target_is_flagged() {
    let findings = findings("wrong-prefix");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("must use prefix @agents"));
}

#[test]
fn two_mappings_already_flagged_skip_prevents_duplicate() {
    let config = config_from_yaml(&format!(
        "{}  - prefix: \"@workers\"\n    root: workers\n",
        agents_config()
    ));
    assert_eq!(check(&fixture("two-mappings"), &config).unwrap().len(), 1);
}

#[test]
fn check_exists_uses_files_and_wildcard_parent_directories() {
    let config = config_from_yaml(&format!("{}checkExists: true\n", agents_config()));
    assert!(check(&fixture("exists-pass"), &config).unwrap().is_empty());
    let findings = check(&fixture("exists-fail"), &config).unwrap();
    assert_eq!(findings.len(), 4, "{findings:?}");
    assert!(findings
        .iter()
        .all(|finding| finding.message.contains("does not exist")));
    // The same absent targets are valid shapes when existence checks are disabled.
    assert!(self::findings("exists-fail").is_empty());
}

#[test]
fn empty_base_dir_no_leading_slash_on_correct_mapping() {
    let config = config_from_yaml(&agents_config().replace("baseDir: backend\n", ""));
    assert!(check(&fixture("empty-base"), &config).unwrap().is_empty());
}

#[test]
fn two_mappings_same_root_accept_directory_and_file_targets() {
    let config = config_from_yaml("tsconfig: tsconfig.json\nbaseDir: backend\nmappings:\n  - prefix: \"@a\"\n    root: shared\n  - prefix: \"@b\"\n    root: shared\n");
    assert!(check(&fixture("shared-root"), &config).unwrap().is_empty());
    let config = config_from_yaml(&format!(
        "{}  - prefix: \"@other\"\n    root: agents\n",
        agents_config()
    ));
    // Both directions must use the same extension-aware matching predicate.
    assert!(check(&fixture("valid-files"), &config).unwrap().is_empty());
}

#[test]
fn inherited_paths_report_winning_file_and_ast_key_lines() {
    let findings = findings("locations");
    let locations: Vec<_> = findings
        .iter()
        .map(|finding| (finding.file.as_str(), finding.line))
        .collect();
    assert_eq!(
        locations,
        vec![
            ("config/policy.base.json", 10),
            ("config/policy.base.json", 11),
            ("config/policy.base.json", 12),
        ]
    );
    assert!(findings[0].message.contains("@agents/email"));
    assert!(findings[1].message.contains("@agents/sms"));
    assert!(findings[2].message.contains("must use prefix"));
}

#[test]
fn child_paths_and_explicit_empty_paths_override_inherited_locations() {
    for name in ["empty.json", "array-empty.json"] {
        let config = config_from_yaml(&agents_config().replace("tsconfig.json", name));
        assert!(check(&fixture("locations"), &config).unwrap().is_empty());
    }
    let config = config_from_yaml(&agents_config().replace("tsconfig.json", "child.json"));
    let findings = check(&fixture("locations"), &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "child.json");
    assert_eq!(findings[0].line, 5);
}

#[test]
fn inherited_findings_filter_on_configured_tsconfig() {
    let root = fixture("locations");
    let mut config = config_from_yaml(agents_config());
    config.rules[0].include = vec!["tsconfig.json".to_string()];
    assert_eq!(check(&root, &config).unwrap().len(), 3);
    config.rules[0].include = vec!["config/*.json".to_string()];
    assert!(check(&root, &config).unwrap().is_empty());
    config.rules[0].include.clear();
    config.rules[0].exclude = vec!["tsconfig.json".to_string()];
    assert!(check(&root, &config).unwrap().is_empty());
}
