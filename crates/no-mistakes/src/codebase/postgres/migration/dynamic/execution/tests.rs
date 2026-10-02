use super::Scope;
use sqlparser::tokenizer::Token;

fn fixture(file: &str) -> String {
    std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-generated-column-writes/unit-fixture/table-event-forms")
            .join(file),
    )
    .unwrap()
}

#[test]
fn immediate_do_returns_keep_unreachable_ddl_out_of_live_history() {
    let facts = crate::codebase::postgres::migration::extract_migration_facts(&fixture(
        "return-termination.sql",
    ));
    let events: Vec<_> = facts
        .table_events
        .iter()
        .map(|event| match event {
            crate::codebase::postgres::SqlTableSchemaEvent::Create { table, .. } => {
                format!("create:{table}")
            }
            crate::codebase::postgres::SqlTableSchemaEvent::AddColumn { column, .. } => {
                format!("add:{}", column.name)
            }
            crate::codebase::postgres::SqlTableSchemaEvent::Drop { table, .. } => {
                format!("drop:{table}")
            }
        })
        .collect();
    assert_eq!(
        events,
        [
            "create:orders",
            "create:quoted_return_kept",
            "add:after_return"
        ]
    );
    // Source-based policy facts still describe unreachable SQL in the block.
    assert!(facts
        .dropped_tables
        .iter()
        .any(|drop| drop.name == "orders"));
    for table in [
        "unreachable_after_return",
        "unreachable_after_nested_return",
        "uncertain_after_conditional_return",
    ] {
        assert!(facts.tables.iter().any(|fact| fact.table_name == table));
    }
}

#[test]
fn only_definite_plain_return_stops_later_statements() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-generated-column-writes/unit-fixture/table-event-forms/return-modes.sql");
    let source = std::fs::read_to_string(path).unwrap();
    let outer = super::super::tokenize(&source);
    let body = outer
        .iter()
        .find_map(|token| match &token.token {
            Token::DollarQuotedString(body) => Some(body.value.as_str()),
            _ => None,
        })
        .unwrap();
    let tokens = super::super::tokenize(body);
    let mut scope = Scope::default();
    let executed = super::super::statements(&tokens)
        .into_iter()
        .map(super::super::significant)
        .filter(|statement| !statement.is_empty())
        .map(|statement| scope.advance(&statement))
        .collect::<Vec<_>>();
    assert_eq!(
        executed,
        [true, true, true, true, true, true, true, false, false, false, false, false, false,]
    );
}
