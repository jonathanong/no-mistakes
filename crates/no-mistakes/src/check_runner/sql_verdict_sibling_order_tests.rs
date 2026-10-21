use super::*;
use no_mistakes::codebase::postgres::SqlVerdictStatus;
use std::path::PathBuf;

fn targets(report: &CheckResults) -> Vec<(&str, usize, Option<&str>)> {
    let mut found: Vec<_> = report
        .rules
        .iter()
        .map(|finding| {
            (
                finding.file.as_str(),
                finding.line,
                finding.target.as_deref(),
            )
        })
        .collect();
    found.sort();
    found
}

#[test]
fn sibling_order_keeps_each_conflict_on_its_own_chunk() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/consumer-sibling-conflict");
    for (config, fail_closed) in [("ignore.yml", false), ("fail.yml", true)] {
        let report = run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(config)),
            None,
            false,
            false,
            true,
        )
        .unwrap();
        let unanalyzable = fail_closed.then_some("unanalyzable-sql");
        let mut expected = vec![
            ("src/before.ts", 7, Some("targetless-arbiter")),
            ("src/exact.ts", 5, Some("targetless-arbiter")),
            ("src/exact.ts", 6, Some("targetless-arbiter")),
            ("src/query.ts", 5, Some("targetless-arbiter")),
            ("src/quoted.ts", 7, Some("targetless-arbiter")),
            ("src/quoted.ts", 12, Some("targetless-arbiter")),
        ];
        if let Some(target) = unanalyzable {
            expected.extend([
                ("src/before.ts", 7, Some(target)),
                ("src/before.ts", 9, Some(target)),
                ("src/before.ts", 11, Some(target)),
                ("src/broken.ts", 4, Some(target)),
            ]);
        }
        expected.sort();
        assert_eq!(targets(&report), expected, "{config}: {:?}", report.rules);
        assert!(report.advisories.is_empty(), "{config}");
        let verdict = |file: &str, nth: usize| {
            let mut rows: Vec<_> = report
                .sql_verdicts
                .iter()
                .filter(|verdict| verdict.file == file)
                .collect();
            rows.sort_by_key(|verdict| verdict.span.0);
            rows[nth]
        };
        let later = verdict("src/query.ts", 0);
        assert_eq!(later.status, SqlVerdictStatus::Violation, "{config}");
        assert!(later.residual_reasons.is_empty(), "{config}");
        let inside = verdict("src/broken.ts", 0);
        assert_eq!(inside.status, SqlVerdictStatus::Abstain, "{config}");
        assert!(inside.residual_reasons.contains(&"invalid-sql".to_string()));
        let targeted_before = verdict("src/before.ts", 0);
        assert_eq!(
            targeted_before.status,
            SqlVerdictStatus::Violation,
            "{config}"
        );
        assert!(targeted_before.residual_reasons.is_empty(), "{config}");
        for nth in [1, 2] {
            let abstained = verdict("src/before.ts", nth);
            assert_eq!(
                abstained.status,
                SqlVerdictStatus::Abstain,
                "{config} {nth}"
            );
            assert!(abstained
                .residual_reasons
                .contains(&"invalid-sql".to_string()));
        }
        for nth in [3, 4] {
            assert_eq!(
                verdict("src/before.ts", nth).status,
                SqlVerdictStatus::ProvenClear,
                "{config} {nth}"
            );
        }
        for nth in [0, 1] {
            let exact = verdict("src/exact.ts", nth);
            assert_eq!(exact.status, SqlVerdictStatus::Violation, "{config} {nth}");
            assert!(exact.residual_reasons.is_empty(), "{config} {nth}");
        }
        let quoted = verdict("src/quoted.ts", 0);
        assert_eq!(quoted.status, SqlVerdictStatus::Violation, "{config}");
        assert!(quoted.residual_reasons.is_empty(), "{config}");
        assert_eq!(
            verdict("src/quoted.ts", 1).status,
            SqlVerdictStatus::ProvenClear,
            "{config}"
        );
        let escaped = verdict("src/quoted.ts", 2);
        assert_eq!(escaped.status, SqlVerdictStatus::Violation, "{config}");
        assert!(escaped.residual_reasons.is_empty(), "{config}");
    }
}
