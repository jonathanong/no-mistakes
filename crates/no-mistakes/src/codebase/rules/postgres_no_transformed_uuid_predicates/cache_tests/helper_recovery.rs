use super::*;

// Output cases moved to cli_postgres_snapshot_transformed_uuid_predicates.

#[test]
fn escaped_interpolation_stays_unknown_while_literal_helper_sql_is_recovered() {
    let mut cfg = config("unanalyzableSql: advise\n");
    cfg.effort = Some(crate::config::v2::Effort::Low);
    let (cfg, files, sources, mut facts) = prepare("low-inputs.ts", cfg);
    facts.enable_sql_verdicts();
    check_with_files_sources_and_facts(&fixture(), &cfg, &files, &sources, &facts).unwrap();
    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 4, "{verdicts:#?}");
    assert_eq!(
        verdicts
            .iter()
            .filter(|verdict| verdict.status == SqlVerdictStatus::Abstain && verdict.advisory)
            .count(),
        2
    );
    assert_eq!(
        verdicts
            .iter()
            .filter(|verdict| verdict.status == SqlVerdictStatus::Violation)
            .count(),
        1
    );
    assert_eq!(
        verdicts
            .iter()
            .filter(|verdict| verdict.status == SqlVerdictStatus::ProvenClear)
            .count(),
        1
    );
}
#[test]
fn low_owner_uncertainty_produces_advice_without_enabling_all_verdicts() {
    let mut cfg = config("unanalyzableSql: advise\n");
    cfg.effort = Some(crate::config::v2::Effort::Low);
    let (cfg, files, sources, facts) = prepare("verdict-inputs.ts", cfg);
    assert!(!facts.wants_sql_verdicts());
    check_with_files_sources_and_facts(&fixture(), &cfg, &files, &sources, &facts).unwrap();
    let records = facts.sql_verdicts();
    assert_eq!(records.len(), 4, "{records:#?}");
    assert!(records.iter().all(|record| record.advisory));
    assert_eq!(
        records
            .iter()
            .filter(|record| record.status == SqlVerdictStatus::Abstain)
            .count(),
        2,
        "{records:#?}"
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.status == SqlVerdictStatus::Violation)
            .count(),
        2,
        "known UUID violations must retain advice for unresolved siblings: {records:#?}"
    );
}
