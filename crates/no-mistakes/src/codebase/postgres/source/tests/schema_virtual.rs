use super::super::{columns, locations::Locations};
use sqlparser::{ast::Statement, dialect::MySqlDialect, parser::Parser};

#[test]
fn typed_generated_column_projection_retains_virtual_storage() {
    let sql = super::fixture("schema-virtual.sql");
    // Exercise the shared AST projection without substituting this dialect in the public API.
    let statements = Parser::parse_sql(&MySqlDialect {}, &sql).unwrap();
    assert_eq!(statements.len(), 1);
    let Statement::CreateTable(table) = &statements[0] else {
        panic!("saved generated-column fixture must create a table");
    };
    let column = columns::column(&table.columns[1], &Locations::new(&sql));
    assert_eq!(column.name.value, "computed_value");
    let generated = column.generated.unwrap();
    assert_eq!(generated.storage.as_deref(), Some("VIRTUAL"));
    assert_eq!(generated.expression.columns[0].parts[0].value, "base_value");
}

#[test]
fn postgres_virtual_grammar_gap_is_reported_instead_of_silently_dropped() {
    let facts = super::facts("schema-virtual.sql");
    assert!(facts.statements.is_empty());
    assert_eq!(facts.diagnostics.len(), 1);
    assert!(facts.diagnostics[0].message.contains("STORED"));
    assert!(facts.diagnostics[0].span.is_some());
}

#[test]
fn constraint_projections_preserve_expression_keys_and_unknown_constraint_sql() {
    let sql = super::fixture("schema-constraint-projections.sql");
    let statements = Parser::parse_sql(&MySqlDialect {}, &sql).unwrap();
    let Statement::CreateTable(table) = &statements[0] else {
        panic!("saved constraint fixture must create a table");
    };
    let locations = Locations::new(&sql);
    assert_eq!(table.constraints.len(), 2);
    let unique = columns::table_constraint(&table.constraints[0], &locations);
    assert_eq!(unique.kind, super::super::PostgresSqlConstraintKind::Unique);
    assert!(
        unique.columns.is_empty(),
        "an expression key is not a plain column"
    );
    assert!(unique.sql.contains("base_value + 1"));
    let index = columns::table_constraint(&table.constraints[1], &locations);
    assert_eq!(index.kind, super::super::PostgresSqlConstraintKind::Other);
    assert!(index.sql.contains("helper_index"));
}
