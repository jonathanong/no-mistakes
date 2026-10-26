use super::run_all_with_sql_verdicts;
use no_mistakes::codebase::postgres::SqlVerdictStatus;
use std::path::PathBuf;

#[test]
fn truncated_append_findings_and_suppression_match_with_ledger_off_or_on() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/truncated-append-parity",
    );
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().to_path_buf();
    for policy in ["report", "advise", "ignore"] {
        let run = |include_sql_verdicts| {
            run_all_with_sql_verdicts(
                root.clone(),
                Some(root.join(format!("{policy}.yml"))),
                None,
                false,
                false,
                include_sql_verdicts,
            )
            .unwrap()
        };
        let plain = run(false);
        let detailed = run(true);
        assert_eq!(plain.rules, detailed.rules, "{policy}");
        assert_eq!(plain.advisories, detailed.advisories, "{policy}");
        assert_eq!(plain.warnings, detailed.warnings, "{policy}");
        assert_eq!(
            plain.rules.len(),
            1,
            "suppressed call leaked: {:?}",
            plain.rules
        );
        assert!(plain.rules[0]
            .message
            .contains("query execution must start"));
        assert!(plain
            .advisories
            .iter()
            .any(|finding| finding.message.contains("variants truncated")));
        assert!(super::json_value(&plain).get("sqlVerdicts").is_none());
        let leading: Vec<_> = detailed
            .sql_verdicts
            .iter()
            .filter(|verdict| verdict.subcheck == "leading-annotation")
            .collect();
        assert_eq!(leading.len(), 2);
        assert!(leading
            .iter()
            .all(|verdict| verdict.status == SqlVerdictStatus::Violation));
        assert!(detailed.sql_verdicts.iter().any(|verdict| {
            verdict.subcheck == "sql-composition"
                && verdict
                    .residual_reasons
                    .iter()
                    .any(|reason| reason.contains("variants truncated"))
        }));
    }
}
