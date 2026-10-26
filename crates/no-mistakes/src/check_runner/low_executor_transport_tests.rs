use super::*;

#[test]
fn low_executor_transports_advice_only_on_actual_outside_contract_calls() {
    let root = crate::test_support::rule_fixture_root("postgres-no-offset")
        .join("low-executor-transports");
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let unknown = marked(&source, "// ADVISE");
    let non_sql = marked(&source, "// NON_SQL");
    for (policy, config) in [
        ("advise", ".no-mistakes-low.yml"),
        ("report", ".no-mistakes-low-report.yml"),
        ("ignore", ".no-mistakes-low-ignore.yml"),
    ] {
        let result = super::super::run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(config)),
            None,
            false,
            false,
            false,
        )
        .unwrap();
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
        for rule in [
            "postgres-no-offset",
            "postgres-no-unbounded-sql-identifier",
            "postgres-no-mutable-sql-registry",
        ] {
            let mut actual = result
                .advisories
                .iter()
                .filter(|row| row.rule == rule)
                .map(|row| row.line)
                .collect::<Vec<_>>();
            // One indirect operation can retain both its call and value escape.
            actual.sort_unstable();
            actual.dedup();
            assert_eq!(
                actual,
                if policy == "advise" {
                    unknown.clone()
                } else {
                    Vec::new()
                },
                "{policy} {rule}: {:?}",
                result.advisories
            );
            assert_eq!(
                actual.iter().filter(|line| non_sql.contains(line)).count(),
                0,
                "known executor transport is not a SQL execution"
            );
            let mut expected = if rule == "postgres-no-offset" {
                marked(&source, "// OFFSET")
            } else {
                Vec::new()
            };
            if policy == "report" {
                expected.extend(&unknown);
            }
            expected.sort_unstable();
            let mut actual = result
                .rules
                .iter()
                .filter(|row| row.rule == rule)
                .map(|row| row.line)
                .collect::<Vec<_>>();
            actual.sort_unstable();
            actual.dedup();
            assert_eq!(actual, expected, "{policy} {rule}: {:?}", result.rules);
        }
    }
}
