use no_mistakes::codebase::rules::RuleFinding;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/low-loop-headers")
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
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains(marker).then_some(index + 1))
        .collect()
}

#[test]
fn loop_headers_use_their_own_execution_count_and_do_not_inherit_body_writes() {
    let source = std::fs::read_to_string(root().join("src/query.mts")).unwrap();
    let missing = marked(&source, "EXPECT_MISSING_");
    let accepted = marked(&source, "EXPECT_ACCEPTED_");

    let report = run(".no-mistakes-low-report.yml");
    assert_eq!(lines_for(&report.rules), missing, "{:?}", report.rules);
    assert!(report.advisories.is_empty(), "{:?}", report.advisories);

    let advise = run(".no-mistakes-low-advise.yml");
    assert_eq!(lines_for(&advise.rules), missing, "{:?}", advise.rules);
    assert!(advise.advisories.is_empty(), "{:?}", advise.advisories);

    let ignore = run(".no-mistakes-low-ignore.yml");
    assert_eq!(lines_for(&ignore.rules), missing, "{:?}", ignore.rules);
    assert!(ignore.advisories.is_empty(), "{:?}", ignore.advisories);

    let max = run(".no-mistakes-max-report.yml");
    for line in accepted {
        assert!(
            !lines_for(&max.rules).contains(&line),
            "max rejected one-time loop header at line {line}: {:?}",
            max.rules
        );
    }
}
