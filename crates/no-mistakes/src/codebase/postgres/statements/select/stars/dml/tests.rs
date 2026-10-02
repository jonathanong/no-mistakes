use super::*;
use sqlparser::dialect::{ClickHouseDialect, GenericDialect, MsSqlDialect};
use sqlparser::parser::Parser;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-explicit-columns/fixture/deferred/sql")
            .join(name),
    )
    .unwrap()
}

#[test]
fn ast_dialect_variants_preserve_base_targets_and_skip_non_tables() {
    let before = fixture("ast-update-before.sql");
    let statements = Parser::parse_sql(&MsSqlDialect {}, &before).unwrap();
    assert_eq!(returning(&before, &statements[0]).len(), 1);
    for file in ["ast-update-nested.sql", "ast-delete-derived.sql"] {
        let sql = fixture(file);
        let statements = Parser::parse_sql(&GenericDialect {}, &sql).unwrap();
        assert!(returning(&sql, &statements[0]).is_empty(), "{file}");
    }
    let sql = fixture("ast-insert-function.sql");
    let statements = Parser::parse_sql(&ClickHouseDialect {}, &sql).unwrap();
    assert!(returning(&sql, &statements[0]).is_empty());
}

#[test]
fn explain_and_delete_ast_forms_share_the_same_projection() {
    let explain = fixture("ast-explain.sql");
    let statements = Parser::parse_sql(&GenericDialect {}, &explain).unwrap();
    assert_eq!(returning(&explain, &statements[0]).len(), 1);
    let joined = fixture("ast-update-join.sql");
    let statements = Parser::parse_sql(&GenericDialect {}, &joined).unwrap();
    assert_eq!(returning(&joined, &statements[0])[0].relation, "orders");
    let sql = fixture("peers.sql");
    let mut statements = Parser::parse_sql(&GenericDialect {}, &sql).unwrap();
    let Statement::Delete(delete) = &mut statements[3] else {
        panic!("fixture delete")
    };
    let FromTable::WithFromKeyword(tables) = &delete.from else {
        panic!("fixture FROM")
    };
    delete.from = FromTable::WithoutKeyword(tables.clone());
    assert_eq!(returning(&sql, &statements[3]).len(), 1);
    let Statement::Delete(delete) = &mut statements[3] else {
        panic!("fixture delete")
    };
    // Empty FROM is only possible in a supplied AST, never a parsed PostgreSQL statement.
    delete.from = FromTable::WithFromKeyword(Vec::new());
    assert!(returning(&sql, &statements[3]).is_empty());
}
