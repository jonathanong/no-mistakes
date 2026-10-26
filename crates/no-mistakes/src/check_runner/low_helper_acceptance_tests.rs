use super::*;
use no_mistakes::codebase::postgres::SqlVerdictStatus;

#[test]
fn all_low_policies_accept_template_and_builder_shapes_at_actual_sql_sites() {
    let root =
        crate::test_support::rule_fixture_root("postgres-no-offset").join("low-helper-recovery");
    let file = "src/acceptance.mts";
    let source = std::fs::read_to_string(root.join(file)).unwrap();
    let result = super::super::run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join(".no-mistakes-low.yml")),
        None,
        false,
        false,
        false,
    )
    .unwrap();
    let clear = marked(&source, "CLEAR_");
    let unknown = marked(&source, "UNKNOWN_");
    let non_sql = marked(&source, "NON_SQL_");
    assert_eq!(clear.len(), 4);
    assert_eq!(unknown.len(), 3);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    for rule in [
        "postgres-no-offset",
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-mutable-sql-registry",
        "postgres-no-transformed-uuid-predicates",
    ] {
        let findings: Vec<_> = result
            .rules
            .iter()
            .filter(|row| row.rule == rule && row.file == file)
            .collect();
        assert!(findings.is_empty(), "{rule}: {findings:?}");
        let advice: Vec<_> = result
            .advisories
            .iter()
            .filter(|row| row.rule == rule && row.file == file)
            .map(|row| row.line)
            .collect();
        assert_eq!(advice, unknown, "{rule}");
        assert!(non_sql.iter().all(|line| !advice.contains(line)));
        for line in &clear {
            // Require a concrete SQL obligation, so dropping a call cannot satisfy silence.
            let verdicts: Vec<_> = result
                .sql_verdicts
                .iter()
                .filter(|row| {
                    row.rule == rule
                        && row.file == file
                        && source[..row.span.1 as usize].lines().count() == *line
                })
                .collect();
            assert_eq!(
                verdicts.len(),
                1,
                "{rule} line {line}: {:?}",
                result.sql_verdicts
            );
            assert_eq!(verdicts[0].status, SqlVerdictStatus::ProvenClear);
            assert!(!verdicts[0].advisory);
        }
    }
}
