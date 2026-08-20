use super::*;
use crate::codebase::postgres::{SqlCreateIndexMetadata, SqlForeignKeyMetadata};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-fk-index/fixture")
            .join(name),
    )
}

fn config(extra: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(&format!(
                "sqlInclude: [\"migrations/**/*.sql\"]\n{extra}"
            ))
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path, extra: &str) -> Vec<RuleFinding> {
    let files = vec![root.join("migrations/001.sql")];
    check_with_files(root, &config(extra), &files).unwrap()
}

#[test]
fn flags_foreign_key_without_leading_index() {
    let findings = run(&fixture("fail"), "");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("comments.post_id"));
    assert!(findings[0].message.contains("btree/hash"));
}

#[test]
fn accepts_btree_leading_index() {
    assert!(run(&fixture("pass"), "").is_empty());
}

#[test]
fn same_line_directive_exempts_the_fk() {
    assert!(run(&fixture("directive"), "").is_empty());
}

#[test]
fn allowed_column_exempts_the_fk() {
    assert!(run(&fixture("allow"), "allowedColumns: [comments.post_id]\n").is_empty());
}

#[test]
fn stale_allowed_column_is_a_finding() {
    let findings = run(&fixture("stale"), "allowedColumns: [comments.missing_id]\n");
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("stale")));
}

#[test]
fn gin_index_does_not_cover_equality_lookups() {
    let findings = run(&fixture("gin"), "");
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn allowed_table_exempts_every_fk_on_that_table() {
    assert!(run(&fixture("fail"), "allowedTables: [comments]\n").is_empty());
}

#[test]
fn stale_allowed_table_is_a_finding() {
    let findings = run(&fixture("pass"), "allowedTables: [missing]\n");
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("allowedTables")));
}

#[test]
fn custom_allow_directive_must_match_the_comment() {
    let findings = run(&fixture("directive"), "allowDirective: skip-fk\n");
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn empty_fk_columns_are_skipped() {
    let opts = compile_options(&Options::default());
    let fk = SqlForeignKeyMetadata {
        table_name: "comments".to_string(),
        column_names: Vec::new(),
        referenced_table_name: "posts".to_string(),
        delete_action: None,
        line: 1,
    };
    let findings = scan::scan_fk(
        "migrations/001.sql",
        "",
        &fk,
        &BTreeMap::new(),
        &opts,
        &mut Default::default(),
        &mut Default::default(),
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn covering_index_shapes() {
    let btree = SqlCreateIndexMetadata {
        table_name: "comments".to_string(),
        leading_column: Some("post_id".to_string()),
        access_method: "btree".to_string(),
        has_predicate: false,
        not_null_predicate_column: None,
    };
    let hash = SqlCreateIndexMetadata {
        access_method: "hash".to_string(),
        ..btree.clone()
    };
    let gin = SqlCreateIndexMetadata {
        access_method: "gin".to_string(),
        ..btree.clone()
    };
    let partial = SqlCreateIndexMetadata {
        has_predicate: true,
        not_null_predicate_column: Some("post_id".to_string()),
        ..btree.clone()
    };
    let other_pred = SqlCreateIndexMetadata {
        has_predicate: true,
        not_null_predicate_column: Some("author_id".to_string()),
        ..btree.clone()
    };
    assert!(scan::covers(&btree, "post_id"));
    assert!(scan::covers(&hash, "POST_ID"));
    assert!(!scan::covers(&gin, "post_id"));
    assert!(scan::covers(&partial, "post_id"));
    assert!(!scan::covers(&other_pred, "post_id"));
    assert!(!scan::covers(&btree, "author_id"));
}

#[test]
fn directive_on_line_ignores_empty_or_missing_lines() {
    assert!(!scan::directive_on_line("", 0, "fk-index-allow"));
    assert!(!scan::directive_on_line("post_id uuid", 1, ""));
    assert!(scan::directive_on_line(
        "post_id uuid REFERENCES posts -- skip-fk",
        1,
        "skip-fk"
    ));
}

#[test]
fn option_defaults_use_empty_includes() {
    let compiled = compile_options(&Options::default());
    assert!(compiled.allowed_columns.is_empty());
    assert!(compiled.allowed_tables.is_empty());
}
