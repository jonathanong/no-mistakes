use super::*;
use no_mistakes::codebase::postgres::SqlVerdictStatus;
use std::collections::BTreeSet;

#[test]
fn helper_patch_grammar_preserves_real_sql_and_residuals_under_every_policy() {
    let root =
        crate::test_support::rule_fixture_root("postgres-no-offset").join("low-helper-recovery");
    let sources = [
        "lower-rejections.mts",
        "evaluation-shapes.mts",
        "carrier-evaluation.mts",
        "patch-catalog.mts",
    ]
    .into_iter()
    .map(|name| {
        let file = format!("src/{name}");
        let source = std::fs::read_to_string(root.join(&file)).unwrap();
        (file, source)
    })
    .collect::<Vec<_>>();
    let unknown = sources
        .iter()
        .flat_map(|(file, source)| {
            marked(source, "ADVISE_")
                .into_iter()
                .map(|line| (file.clone(), line))
        })
        .collect::<BTreeSet<_>>();
    let clear = sources
        .iter()
        .flat_map(|(file, source)| {
            marked(source, "CLEAR_")
                .into_iter()
                .map(|line| (file.clone(), line))
        })
        .collect::<BTreeSet<_>>();
    assert!(unknown.len() >= 20 && clear.len() >= 10);
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
        assert!(result.warnings.is_empty(), "{mode}: {:?}", result.warnings);
        for rule in [
            "postgres-no-offset",
            "postgres-no-unbounded-sql-identifier",
            "postgres-no-mutable-sql-registry",
            "postgres-no-transformed-uuid-predicates",
        ] {
            let rows = |advice: bool| {
                let rows: Vec<_> = if advice {
                    result
                        .advisories
                        .iter()
                        .filter(|row| row.rule == rule)
                        .map(|row| (row.file.clone(), row.line))
                        .collect()
                } else {
                    result
                        .rules
                        .iter()
                        .filter(|row| row.rule == rule)
                        .map(|row| (row.file.clone(), row.line))
                        .collect()
                };
                rows.into_iter()
                    .filter(|(file, _)| sources.iter().any(|(name, _)| name == file))
                    .collect::<BTreeSet<_>>()
            };
            assert_eq!(
                rows(true),
                if mode == "advise" {
                    unknown.clone()
                } else {
                    BTreeSet::new()
                },
                "{mode}/{rule} advice"
            );
            assert_eq!(
                rows(false),
                if mode == "report" {
                    unknown.clone()
                } else {
                    BTreeSet::new()
                },
                "{mode}/{rule} findings"
            );
            for (file, source) in &sources {
                for (wanted, line) in unknown.iter().filter(|(name, _)| name == file) {
                    assert!(
                        result.sql_verdicts.iter().any(|row| row.rule == rule
                            && row.file == *wanted
                            && source[..row.span.1 as usize].lines().count() == *line
                            && ["read(", "query("]
                                .iter()
                                .any(|callee| source[row.span.0 as usize..row.span.1 as usize]
                                    .starts_with(callee))
                            && row.status == SqlVerdictStatus::Abstain),
                        "{mode}/{rule} actual unknown SQL call missing {file}:{line}"
                    );
                }
                for (wanted, line) in clear.iter().filter(|(name, _)| name == file) {
                    let verdicts = result
                        .sql_verdicts
                        .iter()
                        .filter(|row| {
                            row.rule == rule
                                && row.file == *wanted
                                && source[..row.span.1 as usize].lines().count() == *line
                        })
                        .collect::<Vec<_>>();
                    assert!(
                        !verdicts.is_empty(),
                        "{mode}/{rule} missing SQL obligation {file}:{line}"
                    );
                    assert!(verdicts.iter().all(|row| row.status == SqlVerdictStatus::ProvenClear
                        && !row.advisory), "{mode}/{rule} {file}:{line}: {verdicts:?}");
                }
            }
        }
    }
}
