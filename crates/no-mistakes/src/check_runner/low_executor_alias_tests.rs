use super::*;

#[test]
fn low_executor_aliases_prove_only_exact_immutable_identities() {
    let root =
        crate::test_support::rule_fixture_root("postgres-no-offset").join("low-executor-aliases");
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let result = super::super::run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join(".no-mistakes.yml")),
        None,
        false,
        false,
        false,
    )
    .unwrap();
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    for rule in [
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-transformed-uuid-predicates",
    ] {
        let mut actual = result
            .advisories
            .iter()
            .filter(|row| row.rule == rule)
            .map(|row| row.line)
            .collect::<Vec<_>>();
        actual.sort_unstable();
        actual.dedup();
        assert_eq!(
            actual,
            marked(&source, "// ADVISE"),
            "{rule}: {:?}",
            result.advisories
        );
        let actual = result
            .rules
            .iter()
            .filter(|row| row.rule == rule)
            .map(|row| row.line)
            .collect::<Vec<_>>();
        let expected = if rule == "postgres-no-transformed-uuid-predicates" {
            marked(&source, "// TRANSFORMED")
        } else {
            Vec::new()
        };
        assert_eq!(actual, expected, "{rule}: {:?}", result.rules);
    }
}
