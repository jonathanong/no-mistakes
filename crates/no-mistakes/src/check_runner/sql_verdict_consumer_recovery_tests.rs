use super::*;
use no_mistakes::codebase::postgres::{SqlEvidenceKind, SqlVerdictStatus};
use std::path::PathBuf;

#[test]
fn targetless_enum_insert_is_a_proven_finding_in_every_sql_mode() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/consumer-conflict-recovery");
    for mode in ["fail", "ignore", "advise"] {
        let run = |include| {
            run_all_with_sql_verdicts(
                root.clone(),
                Some(root.join(format!("{mode}.yml"))),
                None,
                false,
                false,
                include,
            )
            .unwrap()
        };
        let plain = run(false);
        let detailed = run(true);
        assert_eq!(plain.rules, detailed.rules, "{mode}");
        assert_eq!(plain.advisories, detailed.advisories, "{mode}");
        assert!(plain.advisories.is_empty(), "{mode}");
        assert_eq!(plain.rules.len(), 1, "{mode}: {:?}", plain.rules);
        let finding = &plain.rules[0];
        assert_eq!(finding.rule, "postgres-conflict-ordering", "{mode}");
        assert_eq!(finding.file, "src/query.ts", "{mode}");
        assert_eq!(finding.line, 4, "{mode}");
        assert_eq!(
            finding.target.as_deref(),
            Some("targetless-arbiter"),
            "{mode}"
        );
        let mut with_ledger = json_value(&detailed);
        assert_eq!(detailed.sql_verdicts.len(), 1, "{mode}");
        let verdict = &detailed.sql_verdicts[0];
        assert_eq!(verdict.file, "src/query.ts", "{mode}");
        assert_eq!(verdict.subcheck, "canonical-conflict-order", "{mode}");
        assert_eq!(verdict.status, SqlVerdictStatus::Violation, "{mode}");
        assert_eq!(verdict.evidence, Some(SqlEvidenceKind::Exact), "{mode}");
        assert!(verdict.residual_reasons.is_empty(), "{mode}");
        with_ledger.as_object_mut().unwrap().remove("sqlVerdicts");
        assert_eq!(with_ledger, json_value(&plain), "{mode}");
    }
}

#[test]
fn conflict_inside_executed_cte_emits_the_proven_finding() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/consumer-cte-conflict");
    for config in ["fail.yml", ".no-mistakes.yml", "advise.yml"] {
        let run = |include| {
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
        let plain = run(false);
        let detailed = run(true);
        assert_eq!(plain.rules, detailed.rules, "{config}");
        assert_eq!(plain.advisories, detailed.advisories, "{config}");
        assert!(plain.advisories.is_empty(), "{config}");
        assert_eq!(plain.rules.len(), 1, "{config}: {:?}", plain.rules);
        let finding = &plain.rules[0];
        assert_eq!(finding.file, "src/query.ts", "{config}");
        assert_eq!(finding.line, 4, "{config}");
        assert_eq!(finding.target.as_deref(), Some("missing-canonical-order"));
        assert_eq!(detailed.sql_verdicts.len(), 1, "{config}");
        let verdict = &detailed.sql_verdicts[0];
        assert_eq!(verdict.file, "src/query.ts", "{config}");
        assert_eq!(verdict.status, SqlVerdictStatus::Violation, "{config}");
        assert_eq!(verdict.evidence, Some(SqlEvidenceKind::Exact), "{config}");
        assert!(verdict.residual_reasons.is_empty(), "{config}");
        let mut with_ledger = json_value(&detailed);
        with_ledger.as_object_mut().unwrap().remove("sqlVerdicts");
        assert_eq!(with_ledger, json_value(&plain), "{config}");
    }
}

#[test]
fn cte_conflict_suppression_hides_the_finding_but_keeps_raw_proof() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/consumer-cte-conflict");
    let report = run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join("suppressed.yml")),
        None,
        false,
        false,
        true,
    )
    .unwrap();
    assert!(report.rules.is_empty(), "{:?}", report.rules);
    assert!(report.advisories.is_empty(), "{:?}", report.advisories);
    assert_eq!(report.sql_verdicts.len(), 1);
    let verdict = &report.sql_verdicts[0];
    assert_eq!(verdict.file, "src/suppressed.ts");
    assert_eq!(verdict.status, SqlVerdictStatus::Violation);
    assert_eq!(verdict.evidence, Some(SqlEvidenceKind::Exact));
}

#[test]
fn incomplete_cte_alternative_cannot_prove_a_conflict_violation() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/consumer-cte-conflict");
    for (config, finding) in [
        ("partial.yml", None),
        ("partial-fail.yml", Some("unanalyzable-sql")),
    ] {
        let report = run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(config)),
            None,
            false,
            false,
            true,
        )
        .unwrap();
        assert_eq!(
            report.rules.len(),
            usize::from(finding.is_some()),
            "{config}: {:?}",
            report.rules
        );
        if let Some(target) = finding {
            assert_eq!(report.rules[0].file, "src/partial.ts");
            assert_eq!(report.rules[0].target.as_deref(), Some(target));
        }
        assert_eq!(report.sql_verdicts.len(), 1);
        let verdict = &report.sql_verdicts[0];
        assert_eq!(verdict.file, "src/partial.ts");
        assert_eq!(verdict.status, SqlVerdictStatus::Abstain);
        assert!(verdict
            .residual_reasons
            .contains(&"invalid-sql".to_string()));
    }
}
