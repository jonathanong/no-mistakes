use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/tsconfig-file-coverage/fixture")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// Saved-fixture output cases moved to cli_pkgcfg_snapshot_tsconfig_file_coverage.
// Kept tests use paths discovery will not return, option-string helpers, or an
// include glob that config load rejects before check JSON is rendered.

#[test]
fn javascript_and_node_modules_are_skipped() {
    let root = fixture("fail");
    let files = vec![
        root.join("tsconfig.json"),
        root.join("src/index.ts"),
        root.join("app.js"),
        root.join("node_modules/pkg/index.ts"),
    ];
    let findings = check_with_files(&root, &config("{}"), &files).unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| !finding.file.contains("app.js")
                && !finding.file.contains("node_modules")),
        "{findings:?}"
    );
}

#[test]
fn option_defaults_are_empty() {
    let compiled = compile_options(&Options::default());
    assert!(compiled.allow.is_empty());
    assert!(compiled.auxiliary.is_empty());
    assert!(compiled.invalid.is_empty());
    let allow = ReasonedPath::default();
    assert!(allow.clone().path.is_empty());
    let auxiliary = AuxiliaryConfig::default();
    assert!(auxiliary.clone().path.is_empty());
}

#[test]
fn tsconfig_discovery_matches_basename_pattern() {
    assert!(scan::is_tsconfig_path("tsconfig.json"));
    assert!(scan::is_tsconfig_path("apps/web/tsconfig.app.json"));
    assert!(!scan::is_tsconfig_path("jsconfig.json"));
    assert!(!scan::is_tsconfig_path("node_modules/lib/tsconfig.json"));
    assert!(!scan::is_typescript_path("app.js", Path::new("app.js")));
    assert!(scan::is_typescript_path(
        "src/a.tsx",
        Path::new("src/a.tsx")
    ));
    assert!(!scan::is_typescript_path(
        "index.ts",
        Path::new("/tmp/node_modules/pkg/index.ts")
    ));
}

#[test]
fn normalize_rel_strips_dot_segments_and_rejects_escapes() {
    assert_eq!(normalize_rel("./src/foo.ts").as_deref(), Some("src/foo.ts"));
    assert_eq!(normalize_rel("").as_deref(), Some(""));
    assert_eq!(
        normalize_rel(r"scripts\generate.ts").as_deref(),
        Some("scripts/generate.ts")
    );
    assert_eq!(normalize_rel("/scripts/generate.ts"), None);
    assert_eq!(normalize_rel(r"C:\repo\scripts\generate.ts"), None);
    assert_eq!(normalize_rel("C:/repo/scripts/generate.ts"), None);
    assert_eq!(normalize_rel("../src/foo.ts"), None);
    assert_eq!(normalize_rel("src/../foo.ts"), None);
}

#[test]
fn invalid_rule_include_glob_is_a_configuration_error() {
    let root = fixture("pass");
    let mut cfg = config("{}");
    cfg.rules[0].include = vec!["[".into()];
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let error = check_with_files(&root, &cfg, &files).unwrap_err();
    assert!(error.to_string().contains("include"), "{error}");
}

include!("tests/inventory.rs");
