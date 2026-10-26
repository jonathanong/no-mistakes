use super::*;

pub(super) fn check_package_json(path: &Path, root: &Path) -> Vec<RuleFinding> {
    let sources = crate::codebase::rules::source_store_for_files(&[path.to_path_buf()]);
    check_package_json_with_sources(path, root, &sources)
        .into_iter()
        .map(ClassifiedFinding::into_public)
        .collect()
}

pub(super) fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/package-json-registry-only/fixture")
        .join(path)
}

pub(super) fn config_with_options(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

pub(super) fn pnpm12_rule_root(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/package-json-registry-only")
        .join(name)
}

pub(super) fn issue_1035_root(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/pnpm12-issue-1035")
        .join(name)
}
