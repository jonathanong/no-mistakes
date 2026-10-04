use super::collect_set_inserts;
use crate::codebase::postgres::statements::SqlInsertFact;
use sqlparser::ast::{SetExpr, Statement};

#[test]
fn ignores_set_expression_that_is_not_an_insert_statement() {
    let mut inserts: Vec<SqlInsertFact> = Vec::new();
    let mut insert_n = 0;
    collect_set_inserts(
        "",
        &SetExpr::Insert(Statement::Commit {
            chain: false,
            end: false,
            modifier: None,
        }),
        &mut insert_n,
        &mut inserts,
        None,
    );
    assert!(inserts.is_empty());
    assert_eq!(insert_n, 0);
}
