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
fn reexport_accepts_a_destination_that_matches_a_page() {
    assert!(run("pass-reexport").is_empty());
}

#[test]
fn second_hop_package_and_path_alias_stay_silent() {
    assert!(run("limit-second-hop").is_empty(), "one hop only");
    assert!(run("limit-package").is_empty());
    // #2522: `@/next-shared` is a tsconfig path. This rule follows only a relative
    // default re-export, so the missing destination stays silent.
    assert!(run("limit-path-alias").is_empty());
}
