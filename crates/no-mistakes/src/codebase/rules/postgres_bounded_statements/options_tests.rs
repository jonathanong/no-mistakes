use super::{compile_options, Options};
use crate::codebase::postgres::SqlBoundKind;

fn compile(yaml: &str) -> anyhow::Result<super::CompiledOptions> {
    let options: Options = serde_yaml::from_str(yaml).unwrap();
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
