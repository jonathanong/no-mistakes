use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    branches: Vec<String>,
    exhaustive: bool,
    positive_survives_unknown: bool,
    expected: String,
}

fn parse(value: &str) -> SqlVerdictStatus {
    match value {
        "violation" => SqlVerdictStatus::Violation,
        "proven-clear" => SqlVerdictStatus::ProvenClear,
        "abstain" => SqlVerdictStatus::Abstain,
        _ => panic!("invalid saved verdict case: {value}"),
    }
}

#[test]
fn saved_branch_proofs_preserve_implicit_unknown() {
    let cases: Vec<Case> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-verdicts/fixture/branch-proofs.json"
    )))
    .unwrap();
    for case in cases {
        let branches = case
            .branches
            .iter()
            .map(|value| parse(value))
            .collect::<Vec<_>>();
        assert_eq!(
            combine_sql_verdicts(&branches, case.exhaustive, case.positive_survives_unknown,),
            parse(&case.expected),
            "branches: {:?}, exhaustive: {}",
            case.branches,
            case.exhaustive
        );
    }
}

#[test]
fn patch_gap_dynamic_exact_call_is_a_verified_prefix() {
    let call = EmbeddedSqlCall {
        kind: EmbeddedSqlKind::Dynamic,
        ..Default::default()
    };
    assert_eq!(
        sql_evidence_for_call(&call, SqlVerdictStatus::ProvenClear),
        Some(SqlEvidenceKind::VerifiedPrefix)
    );
}

fn embedded(calls: Vec<EmbeddedSqlCall>) -> crate::codebase::postgres::EmbeddedSqlFileFacts {
    use crate::codebase::postgres::EmbeddedSqlFileFacts;
    let call_spans = vec![(1, 2); calls.len()];
    EmbeddedSqlFileFacts {
        path: std::path::PathBuf::from("query.ts"),
        executor_bindings: Vec::new(),
        calls,
        call_spans,
        exported_helpers: Default::default(),
        exported_programs: Default::default(),
        pending_imported: Vec::new(),
        imported_call_lines: Default::default(),
        source_comment_spans: Vec::new(),
        fragments: Vec::new(),
        fragment_variants: Vec::new(),
        fragment_sites: Vec::new(),
        matched_factory_names: Vec::new(),
        matched_type_names: Vec::new(),
        pending_relative: Default::default(),
    }
}

fn record(
    facts: &crate::codebase::check_facts::CheckFactMap,
    calls: Vec<EmbeddedSqlCall>,
    statements: &[crate::codebase::postgres::SqlStatementFileFacts],
    checks: &[String],
    judge: impl Fn(
        &str,
        &EmbeddedSqlCall,
        &crate::codebase::postgres::SqlStatementFileFacts,
    ) -> SqlBranchJudgment,
) {
    let embedded = embedded(calls);
    record_sql_policy_verdicts(
        SqlPolicyVerdictContext {
            facts,
            embedded: &embedded,
            statements,
        },
        "postgres-rule",
        "query.ts",
        &EmbeddedSqlOptions::default(),
        checks,
        false,
        judge,
    );
}

#[test]
fn record_returns_when_sql_verdicts_were_not_requested() {
    let facts = crate::codebase::check_facts::CheckFactMap::default();
    record(
        &facts,
        Vec::new(),
        &[],
        &["bound".to_string()],
        |_, _, _| SqlBranchJudgment::clear(),
    );
    assert!(facts.sql_verdicts().is_empty());
}

#[test]
fn text_without_prepared_statement_facts_abstains() {
    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    facts.enable_sql_verdicts();
    let call = EmbeddedSqlCall {
        sql_text: Some("SELECT 1".to_string()),
        kind: EmbeddedSqlKind::Inline,
        ..Default::default()
    };
    record(
        &facts,
        vec![call],
        &[],
        &["bound".to_string()],
        |_, _, _| panic!("missing facts are not judged"),
    );
    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 1);
    assert_eq!(verdicts[0].status, SqlVerdictStatus::Abstain);
    assert_eq!(
        verdicts[0].residual_reasons,
        vec!["missing-prepared-sql-facts".to_string()]
    );
}

#[test]
fn unanalyzable_parse_failure_keeps_a_violation_that_survives_unknown() {
    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    facts.enable_sql_verdicts();
    let call = EmbeddedSqlCall {
        kind: EmbeddedSqlKind::Dynamic,
        sql_text: Some("INSERT INTO items (id) VALUES (1)".to_string()),
        ..Default::default()
    };
    let statement = crate::codebase::postgres::SqlStatementFileFacts {
        parse_failed: true,
        ..Default::default()
    };
    record(
        &facts,
        vec![call],
        std::slice::from_ref(&statement),
        &["replay".to_string()],
        |_, _, _| SqlBranchJudgment::violation(true),
    );
    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 1);
    assert_eq!(verdicts[0].status, SqlVerdictStatus::Violation);
    assert!(verdicts[0]
        .residual_reasons
        .iter()
        .any(|reason| reason == "invalid-sql"));
}
