use super::*;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/provenance-advice")
}

#[test]
fn annotation_fail_ignore_and_advise_keep_proven_findings_and_residuals_distinct() {
    let root = fixture();
    let run = |config: &str, include| {
        run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(config)),
            None,
            false,
            false,
            include,
        )
        .unwrap()
    };
    let fail = run(".no-mistakes-fail.yml", false);
    assert_eq!(fail.rules.len(), 1, "{:?}", fail.rules);
    assert!(fail.advisories.is_empty());
    let ignore = run(".no-mistakes-ignore.yml", false);
    assert!(ignore.rules.is_empty(), "{:?}", ignore.rules);
    assert!(ignore.advisories.is_empty());
    let advise = run(".no-mistakes.yml", false);
    assert!(advise.rules.is_empty(), "{:?}", advise.rules);
    assert_eq!(advise.advisories.len(), 1, "{:?}", advise.advisories);
    assert!(advise.advisories[0]
        .message
        .contains("unresolved-leading-sql"));
    assert!(!json_value(&advise)
        .as_object()
        .unwrap()
        .contains_key("sqlVerdicts"));
    let evidence = run(".no-mistakes.yml", true);
    assert_eq!(evidence.rules, advise.rules);
    assert_eq!(evidence.advisories, advise.advisories);
    assert_eq!(
        evidence.sql_verdicts.len(),
        4,
        "{:?}",
        evidence.sql_verdicts
    );
    assert_eq!(
        json_value(&evidence)["sqlVerdicts"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        evidence
            .sql_verdicts
            .iter()
            .filter(|row| row.file != "src/query.ts")
            .count(),
        2
    );
}

#[test]
fn explicit_sql_policy_exemption_clears_without_opaque_advice() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/policy-exempt");
    let report = run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join(".no-mistakes.yml")),
        None,
        false,
        false,
        true,
    )
    .unwrap();
    assert!(report.rules.is_empty(), "{:?}", report.rules);
    assert!(report.advisories.is_empty(), "{:?}", report.advisories);
    assert_eq!(report.sql_verdicts.len(), 2, "{:?}", report.sql_verdicts);
    for row in &report.sql_verdicts {
        assert_eq!(
            row.status,
            no_mistakes::codebase::postgres::SqlVerdictStatus::ProvenClear
        );
        assert_eq!(
            row.evidence,
            Some(no_mistakes::codebase::postgres::SqlEvidenceKind::PolicyExemption)
        );
        assert!(row.residual_reasons.is_empty(), "{row:?}");
    }
}

#[test]
fn unknown_join_peer_blocks_a_missing_predicate_proof() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/unknown-owner");
    let report = run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join(".no-mistakes.yml")),
        None,
        false,
        false,
        true,
    )
    .unwrap();
    let row = report
        .sql_verdicts
        .iter()
        .find(|row| row.subcheck == "relation:orders:column:tenant_id")
        .unwrap();
    assert_eq!(
        row.status,
        no_mistakes::codebase::postgres::SqlVerdictStatus::Abstain
    );
    assert!(row
        .residual_reasons
        .contains(&"unknown-relation-owner".into()));
}

#[test]
fn unknown_target_table_blocks_trigger_replay_clear_on_both_guards() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/unknown-trigger-table");
    let report = run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join(".no-mistakes.yml")),
        None,
        false,
        false,
        true,
    )
    .unwrap();
    let rows = report
        .sql_verdicts
        .iter()
        .filter(|row| row.subcheck == "trigger-replay")
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 2, "{:?}", report.sql_verdicts);
    for row in rows {
        assert_eq!(
            row.status,
            no_mistakes::codebase::postgres::SqlVerdictStatus::Abstain
        );
        assert!(row
            .residual_reasons
            .contains(&"unknown-schema-table".into()));
    }
}

#[test]
fn overlapping_policy_applications_keep_distinct_deterministic_obligations() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/overlapping-applications");
    let run = || {
        run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(".no-mistakes.yml")),
            None,
            false,
            false,
            true,
        )
        .unwrap()
    };
    let report = run();
    let rows = report
        .sql_verdicts
        .iter()
        .filter(|row| row.subcheck == "select-star")
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 2, "{:?}", report.sql_verdicts);
    assert_eq!(rows[0].application.as_ref().unwrap().index, 0);
    assert_eq!(
        rows[0].application.as_ref().unwrap().name.as_deref(),
        Some("strict")
    );
    assert_eq!(
        rows[0].status,
        no_mistakes::codebase::postgres::SqlVerdictStatus::Violation
    );
    assert_eq!(rows[1].application.as_ref().unwrap().index, 1);
    assert_eq!(
        rows[1].application.as_ref().unwrap().name.as_deref(),
        Some("permissive")
    );
    assert_eq!(
        rows[1].status,
        no_mistakes::codebase::postgres::SqlVerdictStatus::ProvenClear
    );
    let expected = json_value(&report)["sqlVerdicts"].clone();
    for _ in 0..4 {
        assert_eq!(json_value(&run())["sqlVerdicts"], expected);
    }
}
