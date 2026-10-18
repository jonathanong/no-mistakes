use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/no-object-mutation-literal/fixture/literal-rewrite.mts",
    )
}

// Spread copies are Unknown. These literals are the rewrites the rule documents.
#[test]
fn literal_rewrite_keeps_a_known_object_and_static_sql() {
    let path = fixture();
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::from_path(&path).unwrap()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );

    let tables = facts
        .value_flow
        .bindings
        .iter()
        .find(|binding| binding.name == "tables")
        .expect("tables binding");
    assert!(tables.invalidated_at.is_none());
    match &tables.value {
        ValueExpression::Object(fields, false) => {
            assert!(fields.contains_key("users"), "{fields:?}");
            assert!(fields.contains_key("orders"), "{fields:?}");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("tables should be a known object, got {other:?}"),
    }

    let returned = facts
        .value_flow
        .returns
        .iter()
        .find_map(|(_, value)| match value {
            ValueExpression::Object(fields, false) if fields.contains_key("client") => Some(fields),
            _ => None,
        })
        .expect("replica return");
    match returned.get("client") {
        Some(ValueExpression::Object(client, false)) => {
            assert!(matches!(client.get("get"), Some(ValueExpression::Named { name, .. }) if name == "get"));
        }
        other => panic!("client should be a known object, got {other:?}"),
    }

    let sql = crate::codebase::postgres::extract_embedded_sql_from_source(
        &path,
        &source,
        &crate::codebase::postgres::EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(
        sql.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM app.users WHERE active")
    );
}
