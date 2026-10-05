use super::{compile_options, Options};
use crate::codebase::postgres::SqlBoundKind;

fn compile(yaml: &str) -> anyhow::Result<super::CompiledOptions> {
    // Rules that scan executor calls must select them; `[]` selects none.
    let options: Options = serde_yaml::from_str(&format!("executorNames: []\n{yaml}")).unwrap();
    compile_options(&options)
}

fn error(yaml: &str) -> String {
    compile(yaml).err().unwrap().to_string()
}

#[test]
fn the_catalog_path_is_required_and_every_statement_kind_is_judged_by_default() {
    assert_eq!(
        error("statements: [select]"),
        "postgres-bounded-statements option schemaCatalogPath: required"
    );
    let compiled = compile("schemaCatalogPath: schema.json").unwrap();
    assert_eq!(
        compiled.statements,
        [
            SqlBoundKind::Select,
            SqlBoundKind::Update,
            SqlBoundKind::Delete
        ]
    );
}

#[test]
fn statements_are_validated_and_case_insensitive() {
    let compiled = compile("schemaCatalogPath: s.json\nstatements: [DELETE, ' select ']").unwrap();
    assert_eq!(
        compiled.statements,
        [SqlBoundKind::Delete, SqlBoundKind::Select]
    );
    let empty = error("schemaCatalogPath: s.json\nstatements: []");
    assert!(
        empty.contains("statements: must name at least one"),
        "{empty}"
    );
    let unknown = error("schemaCatalogPath: s.json\nstatements: [insert]");
    assert!(unknown.contains("unknown statement insert"), "{unknown}");
    let duplicate = error("schemaCatalogPath: s.json\nstatements: [select, SELECT]");
    assert!(duplicate.contains("duplicate entry SELECT"), "{duplicate}");
}

#[test]
fn allow_entries_need_a_reason_a_valid_object_and_no_duplicates() {
    let reasonless = error("schemaCatalogPath: s.json\nallow: [{object: 'table:a', reason: ' '}]");
    assert!(reasonless.contains("needs a reason"), "{reasonless}");
    let invalid = error("schemaCatalogPath: s.json\nallow: [{object: 'a', reason: r}]");
    assert!(invalid.contains("invalid object ref a"), "{invalid}");
    let duplicate = error(
        "schemaCatalogPath: s.json\nallow: [{object: 'table:a', reason: r}, {object: 'table:a', reason: r}]",
    );
    assert!(duplicate.contains("duplicate entry table:a"), "{duplicate}");
}

#[test]
fn unanalyzable_sql_is_validated() {
    let compiled = compile("schemaCatalogPath: s.json\nunanalyzableSql: ignore").unwrap();
    assert!(!compiled.fail_unanalyzable);
    assert!(error("schemaCatalogPath: s.json\nunanalyzableSql: maybe").contains("unanalyzableSql"));
}

#[test]
fn compile_options_reject_an_absent_executor_selection() {
    let error = compile_options(&Options {
        schema_catalog_path: "schema.json".into(),
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
        schema_catalog_path: "schema.json".into(),
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
