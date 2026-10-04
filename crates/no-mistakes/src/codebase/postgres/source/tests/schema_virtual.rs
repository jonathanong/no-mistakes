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
fn postgres_virtual_storage_is_recovered_without_reparsing() {
    let facts = super::facts("schema-virtual.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let super::super::PostgresSqlStatementKind::CreateTable { columns, .. } =
        &facts.statements[0].facts
    else {
        panic!()
    };
    assert_eq!(
        columns[1].generated.as_ref().unwrap().storage.as_deref(),
        Some("VIRTUAL")
    );
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

#[test]
fn generated_storage_preserves_original_modes_and_source_positions() {
    let facts = super::facts("generated-storage.sql");
    assert_eq!(facts.diagnostics.len(), 1);
    assert_eq!(facts.statements.len(), 3);
    let super::super::PostgresSqlStatementKind::CreateTable { columns, .. } =
        &facts.statements[0].facts
    else {
        panic!()
    };
    for column in &columns[1..3] {
        assert_eq!(
            column.generated.as_ref().unwrap().storage.as_deref(),
            Some("VIRTUAL")
        );
    }
    assert_eq!(
        columns[3].generated.as_ref().unwrap().storage.as_deref(),
        Some("STORED")
    );
    assert!(columns[4].generated.is_none());
    assert!(facts.statements[0].sql.contains(") VIRTUAL"));
    let super::super::PostgresSqlStatementKind::AlterTable { operations, .. } =
        &facts.statements[1].facts
    else {
        panic!()
    };
    let super::super::PostgresSqlAlterOperation::AddColumn { column, .. } = &operations[0] else {
        panic!()
    };
    assert_eq!(
        column.generated.as_ref().unwrap().storage.as_deref(),
        Some("VIRTUAL")
    );
    assert_eq!(
        column
            .generated
            .as_ref()
            .unwrap()
            .expression
            .span
            .as_ref()
            .unwrap()
            .start
            .line,
        9
    );
}

#[test]
fn empty_expression_spans_do_not_erase_declared_or_default_virtual_storage() {
    let facts = super::facts("generated-empty.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let super::super::PostgresSqlStatementKind::CreateTable { columns, .. } =
        &facts.statements[0].facts
    else {
        panic!()
    };
    let storage = columns
        .iter()
        .map(|column| column.generated.as_ref().unwrap().storage.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(storage, [Some("VIRTUAL"), Some("STORED"), Some("VIRTUAL")]);
    assert!(columns.iter().all(|column| column
        .generated
        .as_ref()
        .unwrap()
        .expression
        .span
        .is_none()));
    let super::super::PostgresSqlStatementKind::AlterTable { operations, .. } =
        &facts.statements[1].facts
    else {
        panic!()
    };
    let modes = operations
        .iter()
        .filter_map(|operation| {
            if let super::super::PostgresSqlAlterOperation::AddColumn { column, .. } = operation {
                Some(column.generated.as_ref().unwrap().storage.as_deref())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(modes, [Some("VIRTUAL"), Some("STORED"), Some("VIRTUAL")]);
}
