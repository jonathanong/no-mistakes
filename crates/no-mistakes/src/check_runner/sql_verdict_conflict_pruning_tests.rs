use super::*;
use no_mistakes::codebase::postgres::{SqlEvidenceKind, SqlVerdictStatus};
use std::path::PathBuf;

#[test]
fn conflict_pruning_preserves_non_insert_reports_and_cte_findings_in_every_mode() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/conflict-pruning-noninsert");
    let mut expected = None;
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
        let evidence = run(true);
        let plain_json = json_value(&plain);
        let mut evidence_json = json_value(&evidence);
        assert!(plain_json.get("sqlVerdicts").is_none());
        evidence_json.as_object_mut().unwrap().remove("sqlVerdicts");
        assert_eq!(
            plain_json, evidence_json,
            "{mode}: opt-in changed existing fields"
        );
        assert_eq!(plain.rules.len(), 1, "{mode}: {:?}", plain.rules);
        assert_eq!(
            plain.rules[0].target.as_deref(),
            Some("missing-canonical-order")
        );
        assert!(plain.advisories.is_empty());
        if let Some(expected) = &expected {
            assert_eq!(&plain_json, expected, "{mode}: changed policy output");
        } else {
            expected = Some(plain_json);
        }
        assert_eq!(evidence.sql_verdicts.len(), 13);
        assert_eq!(
            evidence
                .sql_verdicts
                .iter()
                .filter(|verdict| verdict.status == SqlVerdictStatus::ProvenClear)
                .count(),
            12,
        );
        assert_eq!(
            evidence
                .sql_verdicts
                .iter()
                .filter(|verdict| verdict.status == SqlVerdictStatus::Violation)
                .count(),
            1,
        );
    }
}

#[test]
fn conflict_pruning_preserves_opaque_tail_reports_in_every_mode() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/conflict-pruning-dynamic");
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
        let evidence = run(true);
        let mut evidence_json = json_value(&evidence);
        evidence_json.as_object_mut().unwrap().remove("sqlVerdicts");
        assert_eq!(json_value(&plain), evidence_json, "{mode}");
        assert_eq!(plain.rules.len(), usize::from(mode == "fail"));
        assert_eq!(plain.advisories.len(), if mode == "advise" { 2 } else { 0 });
        if let Some(finding) = plain.rules.first() {
            assert_eq!(finding.target.as_deref(), Some("unanalyzable-sql"));
            assert_eq!(finding.line, 4);
        }
        assert_eq!(evidence.sql_verdicts.len(), 2);
        for row in evidence.sql_verdicts {
            assert_eq!(row.status, SqlVerdictStatus::Abstain);
            assert_eq!(row.residual_reasons, ["opaque-sql-tail"]);
            assert!(row.known_prefix.is_some());
        }
    }
}

#[test]
fn targetless_arbiter_in_an_opaque_prefix_stays_a_violation() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/conflict-targetless-opaque");
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
        let evidence = run(true);
        let mut evidence_json = json_value(&evidence);
        evidence_json.as_object_mut().unwrap().remove("sqlVerdicts");
        assert_eq!(json_value(&plain), evidence_json, "{mode}");
        assert_eq!(plain.rules.len(), usize::from(mode == "fail") * 2, "{mode}");
        assert_eq!(
            plain.advisories.len(),
            if mode == "advise" { 2 } else { 0 },
            "{mode}"
        );
        assert!(plain
            .rules
            .iter()
            .all(|finding| finding.target.as_deref() == Some("unanalyzable-sql")));
        assert_eq!(evidence.sql_verdicts.len(), 2, "{mode}");
        let violation = evidence
            .sql_verdicts
            .iter()
            .find(|row| row.status == SqlVerdictStatus::Violation)
            .expect("targetless prefix");
        let prefix = violation.known_prefix.as_deref().unwrap_or("");
        assert!(
            prefix.contains("ON CONFLICT DO NOTHING") && !prefix.contains("ON CONFLICT (id)"),
            "{prefix}"
        );
        assert_eq!(violation.evidence, Some(SqlEvidenceKind::VerifiedPrefix));
        assert_eq!(violation.residual_reasons, ["opaque-sql-tail"]);
        let abstain = evidence
            .sql_verdicts
            .iter()
            .find(|row| row.status == SqlVerdictStatus::Abstain)
            .expect("ordered prefix");
        assert!(abstain
            .known_prefix
            .as_deref()
            .unwrap_or("")
            .contains("ON CONFLICT (id)"));
        assert_eq!(abstain.residual_reasons, ["opaque-sql-tail"]);
        assert_eq!(abstain.evidence, None);
    }
}
