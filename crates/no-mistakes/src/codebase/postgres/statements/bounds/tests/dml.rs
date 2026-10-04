use super::super::{dml, Scope, SqlBoundKind};
use sqlparser::{
    ast::{FromTable, Statement, UpdateTableFromKind},
    dialect::{BigQueryDialect, SnowflakeDialect},
    parser::Parser,
};
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/fixture/sql")
            .join(name),
    )
    .unwrap()
}

#[test]
fn update_from_before_set_keeps_the_target_row_cap() {
    let sql = fixture("update-before-set-snowflake.sql");
    let mut statements = Parser::parse_sql(&SnowflakeDialect {}, &sql).unwrap();
    let Statement::Update(update) = statements.remove(0) else {
        panic!("expected UPDATE")
    };
    assert!(matches!(
        update.from,
        Some(UpdateTableFromKind::BeforeSet(_))
    ));

    let mut facts = Vec::new();
    dml::update(&update, &Scope::default(), None, &mut facts);
    assert_eq!(facts[0].kind, SqlBoundKind::Update);
    assert!(facts[0].query.capped);
}

#[test]
fn delete_without_from_keyword_keeps_the_target_row_cap() {
    let sql = fixture("delete-without-from-bigquery.sql");
    let mut statements = Parser::parse_sql(&BigQueryDialect {}, &sql).unwrap();
    let Statement::Delete(delete) = statements.remove(0) else {
        panic!("expected DELETE")
    };
    assert!(matches!(delete.from, FromTable::WithoutKeyword(_)));

    let mut facts = Vec::new();
    dml::delete(&delete, &Scope::default(), None, &mut facts);
    assert_eq!(facts[0].kind, SqlBoundKind::Delete);
    assert!(facts[0].query.capped);
}
