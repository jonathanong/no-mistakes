use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres/conflict-ordering")
}

fn fixture(scenario: &str) -> PathBuf {
    fixture_root().join("cases").join(scenario)
}

fn config() -> NoMistakesConfig {
    config_with_options("schemaCatalogPath: schema.json")
}

fn config_with_options(options: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    });
    config
}

fn files(root: &Path) -> Vec<PathBuf> {
    vec![root.join("src/insert.ts"), root.join("schema.json")]
}

#[test]
fn compile_options_reject_invalid_include_glob_and_unanalyzable_sql() {
    let glob_error = match compile_options(&Options {
        schema_catalog_path: "schema.json".to_string(),
        include: vec!["[".to_string()],
        ..Default::default()
    }) {
        Ok(_) => panic!("invalid include glob should fail"),
        Err(error) => error,
    };
    assert!(glob_error.to_string().contains("include"), "{glob_error:#}");

    let sql_error = match compile_options(&Options {
        schema_catalog_path: "schema.json".to_string(),
        unanalyzable_sql: "maybe".to_string(),
        ..Default::default()
    }) {
        Ok(_) => panic!("invalid unanalyzableSql should fail"),
        Err(error) => error,
    };
    assert!(
        sql_error.to_string().contains("unanalyzableSql"),
        "{sql_error:#}"
    );
}

#[test]
fn compile_options_cover_include_exclude_sql_sources_and_safe_directive() {
    let compiled = compile_options(&Options {
        schema_catalog_path: "schema.json".to_string(),
        include: vec!["src/**/*.ts".to_string()],
        exclude: vec!["src/skip.ts".to_string()],
        sql_include: vec!["queries/**/*.sql".to_string()],
        safe_directive: "SAFE".to_string(),
        unanalyzable_sql: "fail".to_string(),
        ..Default::default()
    })
    .unwrap();
    assert!(compiled.includes("src/insert.ts"));
    assert!(!compiled.includes("src/skip.ts"));
    assert!(!compiled.includes("lib/insert.ts"));
    assert!(compiled.excludes("src/skip.ts"));
    assert!(!compiled.excludes("src/insert.ts"));
    assert_eq!(compiled.safe_directive, "SAFE");
    assert!(compiled.sql_sources.is_some());
    assert!(compiled.fail_unanalyzable);
}

#[test]
fn check_with_files_and_sources_matches_check_with_files() {
    let root = fixture("pass-canonical-order");
    let files = files(&root);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let via_sources = check_with_files_and_sources(&root, &config(), &files, &sources).unwrap();
    assert_eq!(
        via_sources,
        check_with_files(&root, &config(), &files).unwrap()
    );
}

#[test]
fn include_and_sql_exclude_filters_skip_non_matching_paths() {
    let missing = fixture("fail-missing-order");
    let skipped = check_with_files(
        &missing,
        &config_with_options("schemaCatalogPath: schema.json\ninclude: ['src/missing.ts']"),
        &files(&missing),
    )
    .unwrap();
    assert!(skipped.is_empty(), "{skipped:#?}");

    let sql_root = fixture("pass-sql-include");
    let excluded = check_with_files(
        &sql_root,
        &config_with_options(
            "schemaCatalogPath: schema.json\ninclude: ['src/**/*.ts']\nsqlInclude: ['queries/**/*.sql']\nexclude: ['queries/**/*.sql']",
        ),
        &[
            sql_root.join("queries/insert.sql"),
            sql_root.join("schema.json"),
        ],
    )
    .unwrap();
    assert!(excluded.is_empty(), "{excluded:#?}");
}
