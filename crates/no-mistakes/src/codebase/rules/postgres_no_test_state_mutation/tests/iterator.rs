use super::*;

const FILES: &[&str] = &[
    "positive.test.ts",
    "parameter-write.test.ts",
    "escaped-map.test.ts",
    "property-write.test.ts",
    "prototype.test.ts",
    "unknown-rows.test.ts",
    "fake-runner.test.ts",
    "shadowed-runner.test.ts",
    "budget.test.ts",
];

fn root() -> PathBuf {
    fixture().join("effort-iterator")
}

fn run(name: &str) -> (Vec<RuleFinding>, crate::codebase::check_facts::CheckFactMap) {
    let root = root();
    let files = FILES
        .iter()
        .map(|name| root.join("tests").join(name))
        .collect::<Vec<_>>();
    let sources = super::super::super::source_store_for_files(&files);
    let mut config = crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap();
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        &root,
        &mut config,
        &files,
        &sources,
    )
    .unwrap();
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        Arc::clone(&sources),
        &config,
        &[RULE_ID],
    )
    .unwrap()
    .with_fresh_sql_verdicts();
    let findings =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    (findings, facts)
}

#[test]
fn finite_vitest_rows_prove_only_unmodified_local_sql_maps() {
    let root = root();
    let positive = std::fs::read_to_string(root.join("tests/positive.test.ts")).unwrap();
    let origin = positive
        .lines()
        .position(|text| text.contains("iterator DROP origin"))
        .unwrap()
        + 1;
    let (low, facts) = run("low.yml");
    assert!(
        facts.postgres.embedded.is_empty(),
        "low ran max embedded extraction"
    );
    let (max, max_facts) = run("max.yml");
    assert!(
        low.iter().any(|row| {
            row.file == "tests/positive.test.ts"
                && row.line == origin
                && row.target.as_deref() == Some("DROP TABLE")
        }),
        "missing original SQL origin: {low:?}"
    );
    // The max evaluator still reports this computed map as unresolved on this base.
    assert!(max.is_empty(), "unexpected max map finding: {max:?}");
    assert!(
        max_facts.sql_verdicts().iter().any(|row| {
            row.file == "tests/positive.test.ts"
                && row.status == crate::codebase::postgres::SqlVerdictStatus::Abstain
                && row.advisory
        }),
        "max must retain its uncertainty: {:?}",
        max_facts.sql_verdicts()
    );
    let ledger = facts.sql_verdicts();
    let positive_verdict = ledger.iter().find(|row| {
        row.file == "tests/positive.test.ts"
            && row.status == crate::codebase::postgres::SqlVerdictStatus::Violation
    });
    assert!(
        positive_verdict.is_some_and(|row| row.residual_reasons.is_empty()),
        "finite rows remained uncertain: {:?}",
        ledger
    );

    for (file, marker) in [
        ("parameter-write.test.ts", "residual parameter write"),
        ("escaped-map.test.ts", "residual escaped map"),
        ("property-write.test.ts", "residual property write"),
        ("prototype.test.ts", "residual prototype key"),
        ("unknown-rows.test.ts", "residual unknown rows"),
        ("fake-runner.test.ts", "residual fake runner"),
        ("shadowed-runner.test.ts", "residual shadowed runner"),
        ("budget.test.ts", "residual row budget"),
    ] {
        let source = std::fs::read_to_string(root.join("tests").join(file)).unwrap();
        let line = source
            .lines()
            .position(|text| text.contains(marker))
            .unwrap() as u32
            + 1;
        assert!(
            ledger.iter().any(|row| {
                row.file == format!("tests/{file}")
                    && crate::codebase::ts_source::byte_offset_to_line(&source, row.span.0 as usize)
                        == line
                    && row.advisory
                    && !row.residual_reasons.is_empty()
            }),
            "{file}: {:?}",
            ledger
        );
    }
}
