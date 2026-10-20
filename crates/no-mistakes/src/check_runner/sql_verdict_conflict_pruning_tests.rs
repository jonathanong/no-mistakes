use super::*;
use no_mistakes::codebase::postgres::SqlVerdictStatus;
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
        assert_eq!(evidence.sql_verdicts.len(), 3);
        assert_eq!(
            evidence
                .sql_verdicts
                .iter()
                .filter(|verdict| verdict.status == SqlVerdictStatus::ProvenClear)
                .count(),
            2,
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
