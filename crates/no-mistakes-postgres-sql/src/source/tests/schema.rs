use super::super::*;

#[test]
fn column_modifiers_and_return_columns_remain_structured() {
    let facts = facts("schema-variants.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let PostgresSqlStatementKind::CreateTable { columns, .. } = &facts.statements[0].facts else {
        panic!()
    };
    assert!(columns[0].nullable);
    assert_eq!(
        columns[1].constraints[0].kind,
        PostgresSqlConstraintKind::Unique
    );
    assert_eq!(
        columns[2].constraints[0].referenced_columns[0].identity,
        "id"
    );
    assert_eq!(columns[6].data_type.modifiers, ["8"]);
    assert_eq!(columns[7].data_type.modifiers, ["12", "3"]);
    assert_eq!(columns[11].data_type.modifiers, ["20"]);
    assert_eq!(columns[17].data_type.array_dimensions, [Some(4)]);
    let PostgresSqlStatementKind::CreateFunction { function } = &facts.statements[3].facts else {
        panic!()
    };
    assert_eq!(function.return_type.as_ref().unwrap().fields.len(), 2);
    let PostgresSqlStatementKind::AlterTable { operations, .. } = &facts.statements[4].facts else {
        panic!()
    };
    assert!(
        matches!(&operations[0], PostgresSqlAlterOperation::ValidateConstraint { name } if name.identity == "positive_check")
    );
}
use super::facts;

#[test]
fn columns_and_alter_operations_are_typed_without_consumer_ast_dispatch() {
    let facts = facts("schema.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let PostgresSqlStatementKind::CreateTable {
        table,
        columns,
        constraints,
        ..
    } = &facts.statements[0].facts
    else {
        panic!()
    };
    assert_eq!(table.parts[1].identity, "Accounts");
    assert!(!columns[0].nullable);
    assert!(columns[1].data_type.name.as_ref().unwrap().parts[0].quoted);
    assert_eq!(columns[2].data_type.array_dimensions, [None]);
    assert_eq!(columns[3].data_type.array_dimensions, [Some(2), Some(3)]);
    assert!(columns[4].default.is_some());
    assert!(columns[5].identity.is_some());
    assert_eq!(
        columns[6].generated.as_ref().unwrap().storage.as_deref(),
        Some("STORED")
    );
    assert_eq!(
        constraints[2].referenced_table.as_ref().unwrap().parts[1].identity,
        "users"
    );
    let PostgresSqlStatementKind::AlterTable { operations, .. } = &facts.statements[1].facts else {
        panic!()
    };
    assert_eq!(operations.len(), 7);
    assert!(matches!(
        &operations[1],
        PostgresSqlAlterOperation::AlterColumnType { using: Some(_), .. }
    ));
}
