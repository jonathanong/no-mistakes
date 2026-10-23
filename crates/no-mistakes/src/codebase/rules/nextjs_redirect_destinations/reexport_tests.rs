use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/nextjs-redirect-destinations/fixture")
            .join(name),
    )
}

fn config(options_yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(options_yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn run(name: &str) -> Vec<RuleFinding> {
    let root = fixture(name);
    let files = crate::codebase::ts_source::walk_files(&root, &[]);
    check_with_files(&root, &config("{}"), &files).unwrap()
}

#[test]
fn reexport_reports_missing_destinations_in_the_shared_module() {
    let findings = run("fail-reexport");
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings
        .iter()
        .all(|finding| finding.file == "next-shared.ts"));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("redirect destination '/gone'")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("rewrite destination '/missing'")));
}

#[test]
fn default_import_reports_the_shared_module() {
    let findings = run("fail-default-import");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "next-shared.ts");
    assert!(findings[0].message.contains("redirect destination '/gone'"));
}

#[test]
fn exported_default_is_followed_when_another_default_import_is_present() {
    for name in ["fail-earlier-import", "fail-later-import"] {
        let findings = run(name);
        assert_eq!(findings.len(), 1, "{name}: {findings:?}");
        assert_eq!(findings[0].file, "next-shared.ts");
        assert!(findings[0].message.contains("redirect destination '/gone'"));
    }
}

#[test]
fn dotted_basename_without_a_source_extension_resolves() {
    let findings = run("fail-dotted-specifier");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "next.shared.ts");
    assert!(findings[0].message.contains("redirect destination '/gone'"));
}

#[test]
fn string_export_names_follow_the_shared_module() {
    let findings = run("fail-string-export");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "next-shared.ts");
    assert!(findings[0].message.contains("redirect destination '/gone'"));
}

#[test]
fn extensionless_index_and_js_specifier_resolve_the_typescript_module() {
    let index = run("fail-index");
    assert_eq!(index.len(), 1, "{index:?}");
    assert_eq!(index[0].file, "next-shared/index.ts");
    assert!(index[0].message.contains("redirect destination '/gone'"));

    let sibling = run("fail-js-specifier");
    assert_eq!(sibling.len(), 1, "{sibling:?}");
    assert_eq!(sibling[0].file, "next-shared.ts");
    assert!(sibling[0].message.contains("redirect destination '/gone'"));
}

#[test]
fn reexport_accepts_a_destination_that_matches_a_page() {
    assert!(run("pass-reexport").is_empty());
}

#[test]
fn second_hop_and_package_stay_silent() {
    assert!(run("limit-second-hop").is_empty(), "one hop only");
    assert!(run("limit-package").is_empty());
}

#[test]
fn tsconfig_path_alias_reports_the_shared_module() {
    let findings = run("limit-path-alias");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "next-shared.ts");
    assert!(findings[0].message.contains("redirect destination '/gone'"));
}

#[test]
fn path_alias_keeps_one_hop_and_requires_configured_paths() {
    assert!(run("pass-path-alias").is_empty());
    assert!(
        run("limit-alias-precedence").is_empty(),
        "failed exact paths mapping wins over a resolvable wildcard and baseUrl"
    );
    assert!(
        run("limit-missing-exact-alias").is_empty(),
        "failed paths must not admit baseUrl fallback"
    );
    assert!(
        run("limit-missing-wildcard-alias").is_empty(),
        "failed wildcard paths must not admit baseUrl fallback"
    );
    assert!(run("limit-alias-second-hop").is_empty(), "one hop only");
    assert!(
        run("limit-base-url").is_empty(),
        "baseUrl is not a paths alias"
    );
}

#[test]
fn standalone_dispatcher_uses_the_same_alias_resolution() {
    let root = fixture("limit-path-alias");
    let files = crate::codebase::ts_source::walk_files(&root, &[]);
    let findings = super::super::filesystem_dispatch::run_filesystem_rules_with_config(
        &root,
        &config("{}"),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "next-shared.ts");
    assert!(findings[0].message.contains("redirect destination '/gone'"));
}

#[test]
fn disabled_rule_does_not_prepare_a_workspace_or_catalog() {
    let root = fixture("fail-default-import");
    let files = crate::codebase::ts_source::walk_files(&root, &[]);
    let sources = super::super::source_store_for_files(&files);
    let findings =
        check_with_files_and_sources(&root, &NoMistakesConfig::default(), &files, &sources)
            .unwrap();
    assert!(findings.is_empty());
    assert_eq!(
        sources.json_parse_count(),
        0,
        "disabled rule must not parse the malformed manifest"
    );
}
