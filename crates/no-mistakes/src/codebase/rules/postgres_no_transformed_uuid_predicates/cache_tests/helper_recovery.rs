use super::*;

#[test]
fn escaped_interpolation_stays_unknown_while_literal_helper_sql_is_recovered() {
    let mut cfg = config("unanalyzableSql: advise\n");
    cfg.effort = Some(crate::config::v2::Effort::Low);
    let (cfg, files, sources, mut facts) = prepare("low-inputs.ts", cfg);
    facts.enable_sql_verdicts();
    let findings =
        check_with_files_sources_and_facts(&fixture(), &cfg, &files, &sources, &facts).unwrap();
    // The local helper returns literal SQL, so its physical line is now provable.
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 5, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("accounts.id"));
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
    let mut fail = cfg.clone();
    fail.rules[0].resolved_analysis = None;
    fail.rules[0]
        .options
        .as_mapping_mut()
        .unwrap()
        .insert("unanalyzableSql".into(), "fail".into());
    let findings = check_with_files(&fixture(), &fail, &files).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [2, 4, 5],
        "{findings:#?}"
    );
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.target.as_deref() == Some("unanalyzable"))
            .count(),
        2,
        "{findings:#?}"
    );
    assert!(findings
        .iter()
        .any(|finding| { finding.line == 5 && finding.target.as_deref() == Some("accounts.id") }));
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
