use super::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

fn extract(name: &str) -> super::EmbeddedSqlFileFacts {
    let source = std::fs::read_to_string(fixture(name)).expect("fixture");
    extract_embedded_sql_from_source(&fixture(name), &source, &EmbeddedSqlOptions::default())
}

#[test]
fn default_import_from_sql_template_strings_is_inline() {
    let facts = extract("imported-sql-template-strings-default.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Inline);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("INSERT INTO url_content_types (mime_type) VALUES (sql_placeholder_1)")
    );
}

#[test]
fn renamed_default_import_from_sql_template_strings_is_inline() {
    let facts = extract("imported-sql-template-strings-renamed-default.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Inline);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT * FROM topics WHERE id = sql_placeholder_1")
    );
}

#[test]
fn helper_tagged_by_sql_template_strings_default_import_is_composed() {
    let facts = extract("imported-sql-template-strings-helper.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT * FROM topics WHERE id = sql_placeholder_1")
    );
}

#[test]
fn namespace_import_of_sql_template_strings_fails_closed() {
    let facts = extract("imported-sql-template-strings-namespace.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn named_sql_import_from_sql_template_strings_is_inline() {
    let facts = extract("imported-sql-template-strings-named.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Inline);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT * FROM topics WHERE id = sql_placeholder_1")
    );
}

#[test]
fn default_as_import_from_sql_template_strings_is_inline() {
    let facts = extract("imported-sql-template-strings-default-as.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Inline);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT * FROM topics WHERE id = sql_placeholder_1")
    );
}

#[test]
fn default_import_from_an_untrusted_module_fails_closed() {
    let facts = extract("imported-sql-untrusted-default.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn nested_destructure_of_an_imported_sql_tag_alias_fails_closed() {
    let facts = extract("imported-sql-template-strings-nested-destructure.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn named_non_sql_export_aliased_to_sql_fails_closed() {
    let facts = extract("imported-sql-template-strings-named-alias.ts");
    assert_eq!(facts.calls[0].kind, super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn class_function_and_destructured_sql_bindings_fail_closed() {
    let options = EmbeddedSqlOptions::default();
    for source in [
        "class sql {}\nsql`SELECT 1`;",
        "class String {}\nString`SELECT 1`;",
        "function sql() { return 'SELECT 1'; }\nsql`SELECT 1`;",
        "function sql(strings: TemplateStringsArray) { return strings; }\nsql`SELECT 1`;",
        "const { sql } = providers;\nsql`SELECT 1`;",
        "export class sql {}\nsql`SELECT 1`;",
        "export function sql() { return 'SELECT 1'; }\nsql`SELECT 1`;",
        "export interface Sql {}\nexport type SqlAlias = string;\nexport enum Kind { A }\nsql`SELECT 1`;",
        "function sql(strings: TemplateStringsArray): string;\nfunction sql(strings: TemplateStringsArray) { return; }\nsql`SELECT 1`;",
        "function sql(strings: TemplateStringsArray) { foo(); return 'SELECT 1'; }\nsql`SELECT 1`;",
        "export default class {}\nsql`SELECT 1`;",
    ] {
        let facts =
            extract_embedded_sql_from_source(std::path::Path::new("shadow.ts"), source, &options);
        assert!(
            facts.calls.is_empty()
                || facts
                    .calls
                    .iter()
                    .any(|call| call.kind == super::EmbeddedSqlKind::Dynamic),
            "{source}: {facts:#?}"
        );
    }
}
