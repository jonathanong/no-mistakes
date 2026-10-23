use super::run_all_with_sql_verdicts;
use std::path::PathBuf;

#[test]
fn static_append_parse_failures_advise_without_trusting_arbitrary_methods() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/append-static-invalid",
    );
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().to_path_buf();
    for policy in ["report", "advise", "ignore"] {
        let run = |include| {
            run_all_with_sql_verdicts(
                root.clone(),
                Some(root.join(format!("{policy}.yml"))),
                None,
                false,
                false,
                include,
            )
            .unwrap()
        };
        let plain = run(false);
        let detailed = run(true);
        assert_eq!(plain.rules, detailed.rules, "{policy}");
        assert_eq!(plain.advisories, detailed.advisories, "{policy}");
        let lines = plain.rules.iter().map(|row| row.line).collect::<Vec<_>>();
        assert_eq!(
            lines,
            if policy == "report" {
                vec![10, 11, 15]
            } else {
                vec![10, 11]
            },
            "{policy}: {:?} / {:?}",
            plain.rules,
            plain.advisories
        );
        let invalid = plain
            .advisories
            .iter()
            .filter(|row| row.message.contains("invalid composed SQL variant"))
            .map(|row| row.line)
            .collect::<Vec<_>>();
        assert_eq!(
            invalid,
            [4, 5, 8, 9, 19],
            "{policy}: {:?} / {:?}",
            plain.rules,
            plain.advisories
        );
        for advice in plain
            .advisories
            .iter()
            .filter(|row| row.message.contains("invalid composed SQL variant"))
        {
            assert_eq!(
                advice.target.as_deref(),
                Some("sql-composition"),
                "{policy}"
            );
            assert!(
                advice
                    .message
                    .contains("has unresolved SQL for sql-composition"),
                "{policy}: {advice:?}"
            );
            assert!(
                !advice.message.contains("leading-annotation"),
                "{policy}: {advice:?}"
            );
        }
        assert!(plain
            .rules
            .iter()
            .filter(|row| row.line != 15)
            .all(|row| row.message.contains("query execution must start")));
        let rows = detailed
            .sql_verdicts
            .iter()
            .filter(|row| {
                row.subcheck == "leading-annotation"
                    && row
                        .residual_reasons
                        .iter()
                        .any(|reason| reason == "invalid-composed-sql")
            })
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 5, "{policy}");
        assert!(rows.iter().all(|row| row.status
            == no_mistakes::codebase::postgres::SqlVerdictStatus::Abstain
            && !row.advisory
            && row.evidence.is_none()));
        let composition_advice = detailed
            .sql_verdicts
            .iter()
            .filter(|row| row.subcheck == "sql-composition" && row.advisory)
            .collect::<Vec<_>>();
        assert_eq!(composition_advice.len(), 5, "{policy}");
    }
}
