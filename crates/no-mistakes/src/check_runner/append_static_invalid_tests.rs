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
                vec![4, 8, 9, 10, 11, 15, 19]
            } else {
                vec![4, 8, 9, 10, 11, 19]
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
        assert_eq!(rows.len(), 1, "{policy}");
        assert!(rows.iter().all(|row| row.status
            == no_mistakes::codebase::postgres::SqlVerdictStatus::Abstain
            && !row.advisory
            && row.evidence.is_none()));
        assert_eq!(
            detailed
                .sql_verdicts
                .iter()
                .filter(|row| row.subcheck == "leading-annotation"
                    && row.status == no_mistakes::codebase::postgres::SqlVerdictStatus::Violation)
                .count(),
            6
        );
        let composition_advice = detailed
            .sql_verdicts
            .iter()
            .filter(|row| row.subcheck == "sql-composition" && row.advisory)
            .collect::<Vec<_>>();
        assert_eq!(composition_advice.len(), 5, "{policy}");
    }
}

#[test]
fn all_invalid_unnamed_appends_keep_low_max_head_findings_and_suppressions() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/append-invalid-missing-head",
    );
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().to_path_buf();
    for policy in ["ignore", "report", "advise"] {
        let mut findings_by_effort = Vec::new();
        for effort in ["low", "max"] {
            let run = |include| {
                run_all_with_sql_verdicts(
                    root.clone(),
                    Some(root.join(format!("{effort}-{policy}.yml"))),
                    None,
                    false,
                    false,
                    include,
                )
                .unwrap()
            };
            let plain = run(false);
            let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
            let detailed =
                no_mistakes::diagnostics::with_observer(Some(observer.clone()), || run(true));
            assert_eq!(plain.rules, detailed.rules, "{effort}/{policy}");
            assert_eq!(plain.advisories, detailed.advisories, "{effort}/{policy}");
            let expected_lines = if effort == "low" {
                [4, 8, 9, 10]
            } else {
                [4, 5, 9, 10]
            };
            let locations = plain
                .rules
                .iter()
                .map(|row| (row.file.as_str(), row.line))
                .collect::<Vec<_>>();
            eprintln!("observed {effort}/{policy} locations: {locations:?}");
            assert_eq!(
                locations,
                expected_lines.map(|line| ("queries.mts", line)),
                "{effort}/{policy}"
            );
            assert!(plain
                .rules
                .iter()
                .all(|row| row.message.contains("query execution must start")));
            findings_by_effort.push(
                plain
                    .rules
                    .iter()
                    .map(|row| row.message.rsplit(": ").next().unwrap().to_string())
                    .collect::<Vec<_>>(),
            );
            if effort == "max" {
                assert_eq!(
                    plain.advisories.len(),
                    6,
                    "{policy}: {:?}",
                    plain.advisories
                );
                assert!(plain.advisories.iter().all(|row| row.file == "queries.mts"
                    && row.line < 14
                    && row.message.contains("invalid composed SQL variant")));
                assert_eq!(
                    observer.snapshot().work["postgres.sql.variants_skipped_invalid"],
                    10
                );
            } else {
                assert!(plain.advisories.is_empty());
            }
            // Suppressions hide public diagnostics, while raw optional proof
            // still includes the two line opt-outs and the file opt-out.
            assert_eq!(
                detailed
                    .sql_verdicts
                    .iter()
                    .filter(|row| row.subcheck == "leading-annotation"
                        && row.status
                            == no_mistakes::codebase::postgres::SqlVerdictStatus::Violation)
                    .count(),
                7
            );
        }
        assert_eq!(findings_by_effort[0], findings_by_effort[1], "{policy}");
    }
}
