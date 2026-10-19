use super::*;
use crate::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, EmbeddedSqlOptions, ExecutedPostgresSource,
    ExecutedPostgresSql,
};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../test-cases/codebase-analysis/no-object-mutation-literal/fixture/{name}"
    ))
}

fn parsed_facts(path: &Path) -> (String, ImportFacts) {
    let source = std::fs::read_to_string(path).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::from_path(path).unwrap()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    (source, facts)
}

fn executed_sql(path: &Path, source: &str) -> Vec<ExecutedPostgresSql> {
    extract_executed_postgres_sql_with_imports(
        &[ExecutedPostgresSource {
            file_name: path.to_string_lossy().into_owned(),
            source: source.to_string(),
        }],
        &EmbeddedSqlOptions::configured("@example/db", &[]),
        &[],
    )
    .expect("executed sql")
}

// Spread copies are Unknown. These literals are the rewrites the rule documents.
#[test]
fn literal_rewrite_resolves_the_table_in_sql() {
    let path = fixture("literal-rewrite.mts");
    let (source, facts) = parsed_facts(&path);

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
            assert!(
                matches!(client.get("get"), Some(ValueExpression::Named { name, .. }) if name == "get")
            );
        }
        other => panic!("client should be a known object, got {other:?}"),
    }

    let sql = executed_sql(&path, &source);
    assert_eq!(sql.len(), 1, "{sql:?}");
    assert!(sql[0].complete, "{:?}", sql[0]);
    assert!(sql[0].incomplete_reason.is_none(), "{:?}", sql[0]);
    assert_eq!(
        sql[0].sql_text.as_deref(),
        Some("SELECT id FROM app.users WHERE active")
    );
}

#[test]
fn mutated_table_map_stays_unresolved() {
    let path = fixture("literal-mutated.mts");
    let (source, facts) = parsed_facts(&path);
    let tables = facts
        .value_flow
        .bindings
        .iter()
        .find(|binding| binding.name == "tables")
        .expect("tables binding");
    assert!(tables.invalidated_at.is_some(), "{tables:?}");
    match &tables.value {
        ValueExpression::Object(fields, false) => assert!(fields.is_empty(), "{fields:?}"),
        other => panic!("assignment should not add known fields, got {other:?}"),
    }
    let sql = executed_sql(&path, &source);
    assert_eq!(sql.len(), 1, "{sql:?}");
    assert!(!sql[0].complete, "{:?}", sql[0]);
    assert_eq!(sql[0].sql_text, None, "{:?}", sql[0]);
    assert_eq!(
        sql[0].incomplete_reason.as_deref(),
        Some("opaque-executor-operand-effect"),
        "{:?}",
        sql[0]
    );
}

#[test]
fn rows_push_rewrite_is_a_known_array() {
    let (_, facts) = parsed_facts(&fixture("rows-rewrite.mts"));
    let returned = facts
        .value_flow
        .returns
        .iter()
        .find_map(|(_, value)| match value {
            ValueExpression::Array(fields) => Some(fields),
            _ => None,
        })
        .expect("listed return");
    assert!(
        matches!(returned.get("0"), Some(ValueExpression::Named { name, .. }) if name == "row"),
        "{returned:?}"
    );

    let (_, mutated) = parsed_facts(&fixture("rows-mutated.mts"));
    let returned = mutated
        .value_flow
        .returns
        .iter()
        .map(|(_, value)| value)
        .next()
        .expect("add return");
    assert!(
        matches!(returned, ValueExpression::Named { name, .. } if name == "rows"),
        "push leaves the import unresolved, got {returned:?}"
    );
}
