use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-generated-column-writes")
}

fn fixture() -> PathBuf {
    fixture_root().join("fixture")
}

fn unit_fixture(name: &str) -> PathBuf {
    fixture_root().join("unit-fixture").join(name)
}

fn config_with_options(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn fixture_files(root: &Path) -> Vec<PathBuf> {
    vec![
        root.join("schema.sql"),
        root.join("fail-update.ts"),
        root.join("fail-insert-cols.ts"),
        root.join("fail-insert-columnless.ts"),
        root.join("fail-upsert.ts"),
        root.join("fail-merge.sql"),
        root.join("pass.ts"),
    ]
}

#[test]
fn fixture_reports_each_dml_shape_and_skips_source_column_writes() {
    let root = fixture();
    let findings =
        check_with_files(&root, &config_with_options("{}"), &fixture_files(&root)).unwrap();
    let files: Vec<&str> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    assert!(findings.iter().all(|finding| {
        finding.rule == RULE_ID
            && finding.line > 0
            && finding.import.as_deref() == Some("items.created_at")
    }));
    for file in [
        "fail-update.ts",
        "fail-insert-cols.ts",
        "fail-insert-columnless.ts",
        "fail-upsert.ts",
        "fail-merge.sql",
    ] {
        assert!(files.contains(&file), "missing {file}: {findings:#?}");
    }
    assert!(!files.contains(&"pass.ts"));
    assert!(!files.contains(&"schema.sql"));
}

#[test]
fn extra_generated_columns_cover_tables_absent_from_sql() {
    let root = unit_fixture("extra");
    let file = root.join("write.ts");
    let findings = check_with_files(
        &root,
        &config_with_options("extraGeneratedColumns:\n  - table: votes\n    column: created_at\n"),
        &[file],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].import.as_deref(), Some("votes.created_at"));
}

#[test]
fn custom_include_and_executor_bindings() {
    let root = fixture();
    let files = fixture_files(&root);
    let only_update = check_with_files(
        &root,
        &config_with_options("include: ['**/fail-update.ts']"),
        &files,
    )
    .unwrap();
    assert!(
        only_update
            .iter()
            .all(|finding| finding.file == "fail-update.ts"),
        "{only_update:#?}"
    );
    assert!(!only_update.is_empty());

    let custom = unit_fixture("custom-executor");
    let findings = check_with_files(
        &custom,
        &config_with_options(
            "importSpecifier: '@app/db'\nexecutorNames: [run]\nsqlInclude: ['**/schema.sql']\n",
        ),
        &[custom.join("schema.sql"), custom.join("write.ts")],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "write.ts");
}

#[test]
fn invalid_include_glob_errors_and_missing_files_are_skipped() {
    let error = compile_options(&Options {
        include: vec!["[".to_string()],
        ..Options::default()
    })
    .err()
    .expect("invalid include")
    .to_string();
    assert!(error.contains("invalid glob"), "{error}");

    let root = fixture();
    let findings = check_with_files(
        &root,
        &config_with_options("{}"),
        &[root.join("missing.ts")],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn missing_dml_files_with_a_schema_are_silent() {
    let root = fixture();
    let findings = check_with_files(
        &root,
        &config_with_options("{}"),
        &[
            root.join("schema.sql"),
            root.join("missing.ts"),
            root.join("missing.sql"),
        ],
    )
    .unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| finding.file == "schema.sql"),
        "{findings:#?}"
    );
}

#[test]
fn check_entry_point_uses_discovery() {
    let root = fixture();
    let findings = super::check(&root, &config_with_options("{}")).unwrap();
    assert!(
        findings.iter().all(|finding| finding.rule == RULE_ID),
        "{findings:#?}"
    );
}

#[test]
fn empty_catalog_dynamic_sql_and_invalid_schema() {
    let plain = unit_fixture("plain-schema");
    let none = check_with_files(
        &plain,
        &config_with_options("include: ['**/*']"),
        &[
            plain.join("schema.sql"),
            plain.join("write.ts"),
            plain.join("notes.md"),
        ],
    )
    .unwrap();
    assert!(none.is_empty(), "{none:#?}");

    let dynamic = unit_fixture("dynamic");
    let unresolved = check_with_files(
        &dynamic,
        &config_with_options("importSpecifier: ''"),
        &[dynamic.join("schema.sql"), dynamic.join("write.ts")],
    )
    .unwrap();
    assert!(unresolved.is_empty(), "{unresolved:#?}");

    let invalid = unit_fixture("invalid-schema");
    let error = check_with_files(
        &invalid,
        &config_with_options("sqlInclude: ['**/broken.sql']"),
        &[invalid.join("broken.sql")],
    )
    .expect_err("unparseable schema");
    assert!(error.to_string().contains(RULE_ID), "{error:#}");
}

#[test]
fn default_dml_extensions_and_message_shape() {
    assert!(is_default_dml_path(Path::new("a.ts")));
    assert!(is_default_dml_path(Path::new("a.mts")));
    assert!(is_default_dml_path(Path::new("a.tsx")));
    assert!(is_default_dml_path(Path::new("a.js")));
    assert!(is_default_dml_path(Path::new("a.sql")));
    assert!(!is_default_dml_path(Path::new("a.rs")));
    let root = fixture();
    let merge = check_with_files(
        &root,
        &config_with_options("include: ['**/*.sql']"),
        &[root.join("schema.sql"), root.join("fail-merge.sql")],
    )
    .unwrap();
    assert!(
        merge.iter().any(|finding| {
            finding.file == "fail-merge.sql"
                && finding.line >= 1
                && finding.target.as_deref() == Some("created_at")
        }),
        "{merge:#?}"
    );

    let message = finding("src/q.ts", 4, "items", "created_at").message;
    assert!(message.contains("src/q.ts:4"));
    assert!(message.contains("items.created_at"));
    assert!(message.contains("source column"));
}
