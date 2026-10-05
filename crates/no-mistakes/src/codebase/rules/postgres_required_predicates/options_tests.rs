use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-required-predicates/fixture")
            .join(name),
    )
}

fn config_yaml(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn include_exclude_and_option_overrides() {
    let root = fixture("fail");
    let sql = root.join("sql/001.sql");
    let relations = "\nrelations:\n  - table: topics\n    require: [\"parent_id IS NOT NULL\"]";
    assert!(check_with_files(
        &root,
        &config_yaml(&format!(
            "sqlInclude: [\"sql/**/*.sql\"]\nexclude: ['sql/001.sql']{relations}"
        )),
        std::slice::from_ref(&sql),
    )
    .unwrap()
    .is_empty());
    assert_eq!(
        check_with_files(
            &root,
            &config_yaml(&format!(
                "sqlInclude: [\"sql/**/*.sql\"]\ninclude: ['sql/001.sql']{relations}"
            )),
            std::slice::from_ref(&sql),
        )
        .unwrap()
        .len(),
        1
    );
    let error = check_with_files(&root, &config_yaml("exclude: ['[']"), &[sql]).expect_err("glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
    let compiled = compile_options(&Options {
        sql_include: vec!["migrations/**/*.sql".into()],
        import_specifier: Some("@other/db".into()),
        executor_names: Some(vec!["run".into()]),
        unanalyzable_sql: "ignore".into(),
        relations: vec![RelationOption {
            table: "topics".into(),
            require: vec!["parent_id IS NOT NULL".into()],
            ..Default::default()
        }],
        ..Default::default()
    })
    .unwrap();
    assert!(!compiled.fail_unanalyzable);
    assert_eq!(compiled.relations[0].table, "topics");
    assert_eq!(compiled.schema.sql_include, ["migrations/**/*.sql"]);
    assert_eq!(compiled.embedded.import_specifier, "@other/db");
    assert_eq!(compiled.embedded.executor_names, ["run"]);
    assert!(
        compile_options(&Options {
            executor_names: Some(Vec::new()),
            unanalyzable_sql: "fail".into(),
            ..Default::default()
        })
        .unwrap()
        .fail_unanalyzable
    );
}

#[test]
fn dynamic_unparseable_and_unrelated_tables() {
    let dynamic = fixture("fail-dynamic");
    let ts = dynamic.join("src/query.ts");
    let relations = "sqlInclude: [\"sql/**/*.sql\"]\nrelations:\n  - table: topics\n    require: [\"parent_id IS NOT NULL\"]";
    let flagged =
        check_with_files(&dynamic, &config_yaml(relations), std::slice::from_ref(&ts)).unwrap();
    assert!(
        flagged
            .iter()
            .any(|finding| finding.message.contains("not statically recoverable")),
        "{flagged:?}"
    );
    let ignored = check_with_files(
        &dynamic,
        &config_yaml(&format!("unanalyzableSql: ignore\n{relations}")),
        &[ts],
    )
    .unwrap();
    assert!(ignored.is_empty(), "{ignored:?}");
    let root = fixture("fail");
    let unparseable = check_with_files(
        &root,
        &config_yaml(relations),
        &[root.join("sql/unparseable.sql")],
    )
    .unwrap();
    assert!(
        unparseable
            .iter()
            .any(|finding| finding.message.contains("could not be analyzed")),
        "{unparseable:?}"
    );
    let unrelated = compile_options(&Options {
        executor_names: Some(Vec::new()),
        relations: vec![RelationOption {
            table: "accounts".into(),
            require: vec!["id IS NOT NULL".into()],
            ..Default::default()
        }],
        ..Default::default()
    })
    .unwrap();
    let sql = root.join("sql/001.sql");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&sql));
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        std::slice::from_ref(&sql),
        std::sync::Arc::clone(&sources),
        &config_yaml(relations),
        &[RULE_ID],
    )
    .unwrap();
    let findings = scan::scan(
        &root,
        &unrelated,
        std::slice::from_ref(&sql),
        &sources,
        Some(&facts),
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn compile_options_reject_an_absent_executor_selection() {
    let error = compile_options(&Options {
        ..Default::default()
    })
    .err()
    .expect("neither importSpecifier nor executorNames selects an executor");
    assert!(
        error.to_string().starts_with(&format!(
            "{} option importSpecifier: set importSpecifier (or executorNames)",
            super::RULE_ID
        )),
        "{error}"
    );
    // An explicit empty list is the opt-out, not an error.
    let opted_out = compile_options(&Options {
        executor_names: Some(Vec::new()),
        ..Default::default()
    })
    .unwrap();
    assert!(opted_out.embedded.executor_names.is_empty());
}

#[test]
fn standalone_check_rejects_an_absent_executor_selection() {
    // The standalone entry point prepares its facts before compiling options, so
    // the missing executor selection must surface from that preparation too.
    let config = crate::config::v2::NoMistakesConfig {
        rules: vec![crate::config::v2::schema::RuleDef {
            rule: super::RULE_ID.to_string(),
            scope: Some(crate::config::v2::schema::RuleScope::Repository),
            options: serde_yaml::from_str("schemaCatalogPath: schema.json").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let error = super::check_with_files(std::path::Path::new("."), &config, &[])
        .expect_err("neither importSpecifier nor executorNames selects an executor");
    assert!(
        error.to_string().starts_with(&format!(
            "{} option importSpecifier: set importSpecifier (or executorNames)",
            super::RULE_ID
        )),
        "{error}"
    );
}
