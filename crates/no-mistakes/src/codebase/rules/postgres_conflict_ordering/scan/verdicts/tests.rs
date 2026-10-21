use super::super::super::{compile_options, Options};
use super::judge;
use crate::codebase::postgres::{
    EmbeddedSqlCall, EmbeddedSqlKind, SchemaCatalog, SqlBranchJudgment, SqlConflictInsertFact,
    SqlConflictTarget, SqlInsertSourceShape, SqlStatementFileFacts, SqlVerdictStatus,
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

fn catalog() -> SchemaCatalog {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-sql-verdicts/fixture/conflict-pruning-dynamic/schema.json",
    );
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn judged(sql: &str, kind: EmbeddedSqlKind, catalog: &SchemaCatalog) -> SqlBranchJudgment {
    let facts = crate::codebase::postgres::statements::extract_sql_conflict_policy_facts(sql, &[]);
    let call = EmbeddedSqlCall {
        sql_text: Some(sql.to_string()),
        kind,
        ..Default::default()
    };
    judge("query.ts", "", &call, Some(&facts), &options(), catalog)
}

const TARGETLESS: &str = "INSERT INTO items (id) SELECT id FROM pending ON CONFLICT DO NOTHING";
const ORDERED_TARGET: &str =
    "INSERT INTO items (id) SELECT id FROM pending ON CONFLICT (id) DO NOTHING";

#[test]
fn targetless_arbiter_survives_an_unknown_tail() {
    let judgment = judged(
        TARGETLESS,
        EmbeddedSqlKind::Inline,
        &SchemaCatalog::default(),
    );
    assert_eq!(judgment.status, SqlVerdictStatus::Violation);
    assert!(judgment.survives_unknown);
    assert_eq!(judgment.reason, None);
}

#[test]
fn order_finding_does_not_survive_an_unknown_tail() {
    let judgment = judged(ORDERED_TARGET, EmbeddedSqlKind::Inline, &catalog());
    assert_eq!(judgment.status, SqlVerdictStatus::Violation);
    assert!(!judgment.survives_unknown);
}

#[test]
fn mixed_targetless_and_order_findings_do_not_survive_an_unknown_tail() {
    let sql = format!("{TARGETLESS}; {ORDERED_TARGET}");
    let judgment = judged(&sql, EmbeddedSqlKind::Inline, &catalog());
    assert_eq!(judgment.status, SqlVerdictStatus::Violation);
    assert!(!judgment.survives_unknown);
}

fn dynamic(sql: &str, positions: Vec<(u32, u32)>) -> SqlBranchJudgment {
    let call = EmbeddedSqlCall {
        sql_text: Some(sql.to_string()),
        kind: EmbeddedSqlKind::Dynamic,
        recovered_placeholder_positions: positions,
        ..Default::default()
    };
    judge("query.ts", "", &call, None, &options(), &catalog())
}

#[test]
fn opaque_targetless_prefix_is_a_violation() {
    let sql = format!("{TARGETLESS}\nsql_placeholder_1");
    let judgment = dynamic(&sql, vec![(2, 1)]);
    assert_eq!(judgment.status, SqlVerdictStatus::Violation);
    assert!(judgment.survives_unknown);
    assert_eq!(judgment.reason, None);
    // The same prefix is found when recovery left the marker and no position.
    let fallback = dynamic(&sql, Vec::new());
    assert_eq!(fallback.status, SqlVerdictStatus::Violation);
    assert!(fallback.survives_unknown);
}

#[test]
fn opaque_order_prefix_still_abstains() {
    let sql = format!("{ORDERED_TARGET} sql_placeholder_1");
    let judgment = dynamic(&sql, Vec::new());
    assert_eq!(judgment.status, SqlVerdictStatus::Abstain);
    assert!(!judgment.survives_unknown);
    assert_eq!(judgment.reason, Some("opaque-sql-tail"));
}

#[test]
fn opaque_prefix_without_a_proven_targetless_clause_abstains() {
    let missing = EmbeddedSqlCall {
        kind: EmbeddedSqlKind::Dynamic,
        ..Default::default()
    };
    assert_eq!(
        judge(
            "query.ts",
            "",
            &missing,
            None,
            &options(),
            &SchemaCatalog::default(),
        )
        .reason,
        Some("opaque-sql-tail")
    );
    let marker_only = dynamic("sql_placeholder_1", vec![(1, 1)]);
    assert_eq!(marker_only.reason, Some("opaque-sql-tail"));
    let select = dynamic("SELECT id FROM items sql_placeholder_1", Vec::new());
    assert_eq!(select.reason, Some("opaque-sql-tail"));
    let broken = dynamic(
        "INSERT INTO items (id) SELECT id FROM pending ON CONFLICT ( sql_placeholder_1",
        Vec::new(),
    );
    assert_eq!(broken.reason, Some("opaque-sql-tail"));
}
