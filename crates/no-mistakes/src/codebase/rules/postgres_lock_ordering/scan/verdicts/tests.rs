use std::borrow::Cow;

use crate::codebase::postgres::{
    EmbeddedSqlCall, EmbeddedSqlKind, LockingSelectMetadata, SchemaCatalog, SqlBranchJudgment,
    SqlStatementFileFacts, SqlVerdictStatus,
};

fn compiled() -> super::super::super::CompiledOptions {
    super::super::super::compile_options(&super::super::super::Options::default()).unwrap()
}

fn lock(table: &str) -> LockingSelectMetadata {
    LockingSelectMetadata {
        has_multi_row_predicate: true,
        has_order_by: false,
        skips_locked_rows: false,
        tables: Some(vec![table.to_string()]),
        table_qualifiers: None,
        order: None,
        pinned_columns: None,
        join_equalities: Vec::new(),
    }
}

fn call(sql_text: Option<&str>) -> EmbeddedSqlCall {
    EmbeddedSqlCall {
        line: 1,
        kind: EmbeddedSqlKind::Inline,
        sql_text: sql_text.map(str::to_string),
        ..Default::default()
    }
}

fn abstains(judgment: SqlBranchJudgment, reason: &str) {
    assert_eq!(judgment.status, SqlVerdictStatus::Abstain);
    assert_eq!(judgment.reason, Some(reason));
}

#[test]
fn unresolved_sql_has_no_lock_metadata() {
    let error = super::metadata(&call(None), None).unwrap_err();
    assert_eq!(error, "unresolved-sql-fragment");
}

#[test]
fn a_parsed_statement_without_variant_locations_has_no_prepared_facts() {
    let statement = SqlStatementFileFacts::default();
    let error = super::metadata(
        &call(Some("SELECT id FROM items FOR UPDATE")),
        Some(&statement),
    )
    .unwrap_err();
    assert_eq!(error, "missing-prepared-sql-facts");
}

#[test]
fn textless_sql_without_a_lock_clause_abstains() {
    let metadata: Result<Cow<'_, [LockingSelectMetadata]>, &'static str> = Err("unused");
    abstains(
        super::judge(
            "row-lock-order",
            "",
            &call(None),
            &compiled(),
            None,
            &metadata,
        ),
        "unresolved-sql-fragment",
    );
}

#[test]
fn catalog_key_order_abstains_without_a_catalog() {
    let locks = vec![lock("items")];
    let metadata = Ok(Cow::Owned(locks));
    abstains(
        super::judge(
            "catalog-key-order",
            "",
            &call(Some("SELECT id FROM items FOR UPDATE")),
            &compiled(),
            None,
            &metadata,
        ),
        "missing-relation-catalog",
    );
}

#[test]
fn an_interpolated_relation_that_is_in_the_catalog_is_still_unknown() {
    let catalog = SchemaCatalog::from_json(
        r#"{
            "formatVersion": 2,
            "coverage": "complete",
            "tables": {
                "sql_placeholder_1": {
                    "relationKind": "table",
                    "columns": {"id": {"dataType": "text"}},
                    "primaryKey": {"columns": ["id"]},
                    "indexes": {}
                }
            }
        }"#,
    )
    .unwrap();
    let locks = vec![lock("sql_placeholder_1")];
    let metadata = Ok(Cow::Owned(locks));
    abstains(
        super::judge(
            "catalog-key-order",
            "",
            &call(Some("SELECT id FROM sql_placeholder_1 FOR UPDATE")),
            &compiled(),
            Some(&catalog),
            &metadata,
        ),
        "unknown-locked-relation",
    );
}
