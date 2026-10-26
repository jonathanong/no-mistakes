use super::*;
use no_mistakes::codebase::postgres::SqlVerdictStatus;

#[test]
fn offset_execution_ownership_preserves_sites_tokens_scopes_and_suppression() {
    let root =
        crate::test_support::rule_fixture_root("postgres-no-offset").join("low-helper-recovery");
    let sources = ["execution-ownership.mts", "execution-ownership-helper.mts"]
        .into_iter()
        .map(|name| {
            let file = format!("src/{name}");
            let source = std::fs::read_to_string(root.join(&file)).unwrap();
            (file, source)
        })
        .collect::<Vec<_>>();
    let known = sources
        .iter()
        .flat_map(|(file, source)| {
            source.lines().enumerate().flat_map(move |(i, line)| {
                let count = if line.contains("EXPECT_OFFSET_TWICE") {
                    2
                } else {
                    usize::from(line.contains("EXPECT_OFFSET"))
                };
                std::iter::repeat_n((file.clone(), i + 1), count)
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(known.len(), 11);
    let unknown = sources
        .iter()
        .flat_map(|(file, source)| {
            marked(source, "EXPECT_UNKNOWN")
                .into_iter()
                .map(|line| (file.clone(), line))
        })
        .collect::<Vec<_>>();
    assert_eq!(unknown.len(), 1);
    let selected = |file: &str| sources.iter().any(|(name, _)| name == file);
    for (mode, config) in [
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
            true,
        )
        .unwrap();
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
        let mut actual = result
            .rules
            .iter()
            .filter(|row| row.rule == "postgres-no-offset" && selected(&row.file))
            .map(|row| (row.file.clone(), row.line))
            .collect::<Vec<_>>();
        let mut expected = known.clone();
        if mode == "report" {
            expected.extend(unknown.clone());
        }
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected, "{mode}: {:?}", result.rules);
        let advice = result
            .advisories
            .iter()
            .filter(|row| row.rule == "postgres-no-offset" && selected(&row.file))
            .map(|row| (row.file.clone(), row.line))
            .collect::<Vec<_>>();
        assert_eq!(
            advice,
            if mode == "advise" {
                unknown.clone()
            } else {
                vec![]
            }
        );
        for (file, source) in &sources {
            for (i, line) in source.lines().enumerate().filter(|(_, line)| {
                line.contains("EXPECT_KNOWN_CALL") || line.contains("EXPECT_CALLER_PROOF")
            }) {
                let verdicts = result
                    .sql_verdicts
                    .iter()
                    .filter(|row| {
                        row.rule == "postgres-no-offset"
                            && row.file == *file
                            && no_mistakes::codebase::ts_source::byte_offset_to_line(
                                source,
                                row.span.0 as usize,
                            ) as usize
                                == i + 1
                    })
                    .collect::<Vec<_>>();
                assert!(!verdicts.is_empty(), "{mode}/{file}:{} {line}", i + 1);
                assert!(
                    verdicts
                        .iter()
                        .all(|row| row.status == SqlVerdictStatus::Violation),
                    "{mode}/{file}:{} {verdicts:?}",
                    i + 1
                );
            }
        }
    }
    // An unrelated application's included helper must not cover this excluded owner.
    let narrow = super::super::run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join(".no-mistakes-owned-caller.yml")),
        None,
        false,
        false,
        true,
    )
    .unwrap();
    assert!(narrow.warnings.is_empty());
    let mut expected = known
        .into_iter()
        .filter(|(file, _)| file == "src/execution-ownership.mts")
        .collect::<Vec<_>>();
    let mut actual = narrow
        .rules
        .iter()
        .filter(|row| selected(&row.file))
        .map(|row| (row.file.clone(), row.line))
        .collect::<Vec<_>>();
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected, "{:?}", narrow.rules);
    let caller = &sources[0];
    let foreign_line = marked(&caller.1, "FOREIGN_REPLAY")[0];
    let mut expected_advice = unknown;
    expected_advice.push((caller.0.clone(), foreign_line));
    expected_advice.sort();
    let mut advice = narrow
        .advisories
        .iter()
        .filter(|row| selected(&row.file))
        .map(|row| (row.file.clone(), row.line))
        .collect::<Vec<_>>();
    advice.sort();
    assert_eq!(advice, expected_advice);
    assert!(narrow.sql_verdicts.iter().any(|row| row.file == caller.0
        && no_mistakes::codebase::ts_source::byte_offset_to_line(&caller.1, row.span.0 as usize)
            as usize
            == foreign_line
        && row.status == SqlVerdictStatus::Abstain));
}
