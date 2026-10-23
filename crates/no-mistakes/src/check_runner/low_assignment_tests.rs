use no_mistakes::codebase::rules::RuleFinding;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/low-assignments")
}

fn run(config: &str) -> crate::check_runner::CheckResults {
    let root = root();
    super::run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join(config)),
        None,
        false,
        false,
        false,
    )
    .unwrap_or_else(|error| panic!("{config}: {error}"))
}

fn lines_for(rows: &[RuleFinding]) -> Vec<usize> {
    let mut lines = rows
        .iter()
        .filter(|row| row.file == "src/query.mts")
        .map(|row| row.line)
        .collect::<Vec<_>>();
    lines.sort_unstable();
    lines
}

fn marked(source: &str, marker: &str) -> Vec<usize> {
    let mut lines = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains(marker).then_some(index + 1))
        .collect::<Vec<_>>();
    lines.sort_unstable();
    lines
}

#[test]
fn mutable_literal_assignments_are_decided_at_low_and_keep_unknowns_separate() {
    let source = std::fs::read_to_string(root().join("src/query.mts")).unwrap();
    let missing = marked(&source, "EXPECT_MISSING_");
    let accepted = marked(&source, "EXPECT_ACCEPTED_");
    let unknown = marked(&source, "EXPECT_UNKNOWN_");
    let suppressed = marked(&source, "EXPECT_SUPPRESSED_");

    let report = run(".no-mistakes-low-report.yml");
    let mut expected_report = missing.clone();
    expected_report.extend(unknown.iter().copied());
    expected_report.sort_unstable();
    assert_eq!(
        lines_for(&report.rules),
        expected_report,
        "{:?}",
        report.rules
    );
    assert!(report.advisories.is_empty(), "{:?}", report.advisories);

    let advise = run(".no-mistakes-low-advise.yml");
    assert_eq!(lines_for(&advise.rules), missing, "{:?}", advise.rules);
    assert_eq!(
        lines_for(&advise.advisories),
        unknown,
        "{:?}",
        advise.advisories
    );

    let ignore = run(".no-mistakes-low-ignore.yml");
    assert_eq!(lines_for(&ignore.rules), missing, "{:?}", ignore.rules);
    assert!(ignore.advisories.is_empty(), "{:?}", ignore.advisories);

    for rows in [&report.rules, &advise.rules, &ignore.rules] {
        let actual = lines_for(rows);
        assert!(
            accepted.iter().all(|line| !actual.contains(line)),
            "{:?}",
            rows
        );
        assert!(
            suppressed.iter().all(|line| !actual.contains(line)),
            "{:?}",
            rows
        );
    }

    let max = run(".no-mistakes-max-report.yml");
    // Max attributes finite alternatives to their SQL literals; low reports
    // the executor. These four literal locations represent the same three calls.
    for line in marked(&source, "EXPECT_MAX_MISSING") {
        assert!(
            lines_for(&max.rules).contains(&line),
            "max lost definite literal violation at line {line}: {:?}",
            max.rules
        );
    }
}
