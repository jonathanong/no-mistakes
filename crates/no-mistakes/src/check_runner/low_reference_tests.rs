use no_mistakes::codebase::rules::RuleFinding;
use std::path::PathBuf;
fn marked(source: &str, marker: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter_map(|(i, line)| line.contains(marker).then_some(i + 1))
        .collect()
}
fn lines(rows: &[RuleFinding]) -> Vec<usize> {
    rows.iter().map(|row| row.line).collect()
}
#[test]
fn literal_scalar_reads_stay_decidable_and_tagged_assignment_results_cannot_escape() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/low-reference-uses",
    );
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let missing = marked(&source, "EXPECT_MISSING_");
    let unknown = marked(&source, "EXPECT_UNKNOWN_");
    for policy in ["report", "advise", "ignore"] {
        let result = super::run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(format!(".no-mistakes-low-{policy}.yml"))),
            None,
            false,
            false,
            false,
        )
        .unwrap();
        let mut expected = missing.clone();
        if policy == "report" {
            expected.extend(&unknown);
            expected.sort_unstable();
        }
        assert_eq!(
            lines(&result.rules),
            expected,
            "{policy}: {:?}",
            result.rules
        );
        assert_eq!(
            lines(&result.advisories),
            if policy == "advise" {
                unknown.clone()
            } else {
                Vec::new()
            },
            "{policy}"
        );
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    }
}
