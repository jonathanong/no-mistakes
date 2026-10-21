use super::super::super::{compile_options, Options};
use super::judge;
use crate::codebase::postgres::{
    EmbeddedSqlCall, EmbeddedSqlKind, SchemaCatalog, SqlConflictInsertFact, SqlConflictTarget,
    SqlInsertSourceShape, SqlStatementFileFacts, SqlVerdictStatus,
};

fn options() -> super::super::super::CompiledOptions {
    compile_options(&Options {
        schema_catalog_path: "schema.json".to_string(),
        ..Default::default()
    })
    .expect("default conflict options")
}

fn insert_call() -> EmbeddedSqlCall {
    EmbeddedSqlCall {
        sql_text: Some("INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING".to_string()),
        kind: EmbeddedSqlKind::Inline,
        ..Default::default()
    }
}

fn reason(
    call: &EmbeddedSqlCall,
    prepared: Option<&SqlStatementFileFacts>,
) -> Option<&'static str> {
    judge(
        "query.ts",
        "",
        call,
        prepared,
        &options(),
        &SchemaCatalog::default(),
    )
    .reason
}

#[test]
fn missing_sql_text_abstains() {
    let call = EmbeddedSqlCall {
        kind: EmbeddedSqlKind::Inline,
        ..Default::default()
    };
    assert_eq!(reason(&call, None), Some("unresolved-sql-fragment"));
}

#[test]
fn missing_prepared_facts_abstain() {
    assert_eq!(
        reason(&insert_call(), None),
        Some("missing-prepared-sql-facts")
    );
}

#[test]
fn empty_conflicts_are_unresolved() {
    let facts = SqlStatementFileFacts {
        variant_locations: Some(Default::default()),
        ..Default::default()
    };
    assert_eq!(
        reason(&insert_call(), Some(&facts)),
        Some("unresolved-conflict-statement")
    );
}

#[test]
fn overlapping_parse_failure_abstains_after_reliable_conflicts() {
    let mut facts = SqlStatementFileFacts {
        parse_failed: true,
        variant_locations: Some(Default::default()),
        ..Default::default()
    };
    {
        let locations = facts.variant_locations.as_mut().expect("locations");
        locations.conflict_failure_overlaps = true;
        locations.reliable_conflicts.push(SqlConflictInsertFact {
            table: "items".to_string(),
            target: SqlConflictTarget::Targetless,
            source: SqlInsertSourceShape::default(),
        });
    }
    let judgment = judge(
        "query.ts",
        "",
        &insert_call(),
        Some(&facts),
        &options(),
        &SchemaCatalog::default(),
    );
    assert_eq!(judgment.status, SqlVerdictStatus::Abstain);
    assert_eq!(judgment.reason, Some("invalid-sql"));
}

#[test]
fn missing_prepared_conflict_locations_abstain() {
    let sql = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/conflict-location-demand.sql"),
    )
    .unwrap();
    let call = EmbeddedSqlCall {
        sql_text: Some(sql.clone()),
        kind: EmbeddedSqlKind::Inline,
        ..Default::default()
    };
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    let judgment = judge(
        "query.ts",
        "",
        &call,
        Some(&facts),
        &options(),
        &SchemaCatalog::default(),
    );
    assert_eq!(judgment.status, SqlVerdictStatus::Abstain);
    assert_eq!(judgment.reason, Some("missing-prepared-sql-facts"));
}
