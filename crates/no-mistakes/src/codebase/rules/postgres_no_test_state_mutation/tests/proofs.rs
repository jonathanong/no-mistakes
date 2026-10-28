use super::*;

fn run(name: &str) -> (Vec<RuleFinding>, crate::codebase::check_facts::CheckFactMap) {
    let root = fixture().join("effort-proofs");
    let files = [
        root.join("tests/proofs.test.ts"),
        root.join("helpers/sql.ts"),
    ];
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
fn bounded_helpers_keep_definite_mutations_and_residual_uncertainty() {
    let root = fixture().join("effort-proofs");
    let path = root.join("tests/proofs.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let helper_source = std::fs::read_to_string(root.join("helpers/sql.ts")).unwrap();
    let line = |marker: &str| {
        source
            .lines()
            .position(|text| text.contains(marker))
            .unwrap() as u32
            + 1
    };
    let (low, facts) = run("low.yml");
    assert!(
        facts.postgres.embedded.is_empty(),
        "low ran max embedded extraction"
    );
    let (max, _) = run("max.yml");
    for target in ["DROP TABLE", "pg_sleep"] {
        assert!(
            low.iter().any(|row| row.target.as_deref() == Some(target)),
            "{target}: {low:?}"
        );
        assert!(
            max.iter().any(|row| row.target.as_deref() == Some(target)),
            "{target}: {max:?}"
        );
    }
    let throw_origin = helper_source
        .lines()
        .position(|text| text.contains("throw effect DROP origin"))
        .unwrap()
        + 1;
    assert!(
        low.iter().any(|row| {
            row.file == "helpers/sql.ts"
                && row.line == throw_origin
                && row.target.as_deref() == Some("DROP TABLE")
        }),
        "throw argument SQL finding lost: {low:?}"
    );
    for (marker, expected) in [
        (
            "recovered safe helper",
            crate::codebase::postgres::SqlVerdictStatus::ProvenClear,
        ),
        (
            "recovered DROP helper",
            crate::codebase::postgres::SqlVerdictStatus::Violation,
        ),
        (
            "recovered sleep helper",
            crate::codebase::postgres::SqlVerdictStatus::Violation,
        ),
    ] {
        assert!(
            facts.sql_verdicts().iter().any(|row| {
                crate::codebase::ts_source::byte_offset_to_line(&source, row.span.0 as usize)
                    == line(marker)
                    && row.status == expected
                    && row.evidence.is_some()
                    && row.residual_reasons.is_empty()
            }),
            "{marker}: {:?}",
            facts.sql_verdicts()
        );
    }
    for marker in [
        "uncertain helper branch",
        "invalid helper SQL",
        "exported mutable SQL",
        "external wrapper",
        "bounded depth residual",
        "escaped helper",
        "throw argument has SQL effect",
    ] {
        assert!(
            facts.sql_verdicts().iter().any(|row| {
                crate::codebase::ts_source::byte_offset_to_line(&source, row.span.0 as usize)
                    == line(marker)
                    && row.advisory
                    && row.status == crate::codebase::postgres::SqlVerdictStatus::Abstain
            }),
            "{marker}: {:?}",
            facts.sql_verdicts()
        );
    }
}
