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
            options: serde_yaml::from_str(yaml).unwrap(),
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
        import_specifier: "@other/db".into(),
        executor_names: vec!["run".into()],
        unanalyzable_sql: "ignore".into(),
        relations: vec![RelationOption {
            table: "topics".into(),
            require: vec!["parent_id IS NOT NULL".into()],
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
        relations: vec![RelationOption {
            table: "accounts".into(),
            require: vec!["id IS NOT NULL".into()],
        }],
        ..Default::default()
    })
    .unwrap();
    let sql = root.join("sql/001.sql");
    let findings = scan::scan(
        &root,
        &unrelated,
        std::slice::from_ref(&sql),
        &super::super::source_store_for_files(std::slice::from_ref(&sql)),
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}
