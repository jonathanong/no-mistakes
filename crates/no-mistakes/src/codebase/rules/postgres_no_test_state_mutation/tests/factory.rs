use super::*;

#[test]
fn declared_factory_forwarding_keeps_opaque_advice_and_known_helper_mutation() {
    let root = fixture().join("effort-forwarding");
    let files = [
        root.join("tests/forwarding.test.ts"),
        root.join("db/index.ts"),
    ];
    let source = std::fs::read_to_string(&files[0]).unwrap();
    let sources = super::super::super::source_store_for_files(&files);
    let mut config = crate::config::v2::load_v2_config(&root, Some(&root.join("low.yml"))).unwrap();
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
    let rows = facts.sql_verdicts();
    let line = |marker: &str| {
        source
            .lines()
            .position(|text| text.contains(marker))
            .unwrap() as u32
            + 1
    };
    for marker in [
        "first executor forwarding",
        "second executor forwarding",
        "unresolved selected sink",
    ] {
        assert!(
            rows.iter().any(|row| {
                row.file == "tests/forwarding.test.ts"
                    && crate::codebase::ts_source::byte_offset_to_line(&source, row.span.0 as usize)
                        == line(marker)
                    && row.status == crate::codebase::postgres::SqlVerdictStatus::Abstain
                    && row.advisory
            }),
            "{marker}: {rows:?}"
        );
    }
    let mutation = line("helper executes a known mutation");
    assert!(
        rows.iter().any(|row| {
            row.file == "tests/forwarding.test.ts"
                && crate::codebase::ts_source::byte_offset_to_line(&source, row.span.0 as usize)
                    == mutation
                && row.status == crate::codebase::postgres::SqlVerdictStatus::Violation
                && row.residual_reasons.is_empty()
        }),
        "known helper mutation lost: {rows:?}"
    );
    assert!(
        findings.iter().any(|row| {
            row.file == "tests/forwarding.test.ts"
                && row.line == line("DROP TABLE app.forwarded_mutation") as usize
                && row.target.as_deref() == Some("DROP TABLE")
        }),
        "known helper mutation finding lost: {findings:?}"
    );
}

#[test]
fn declared_relative_factory_creates_real_low_sql_obligations() {
    let root = fixture().join("effort-factory");
    let files = [
        root.join("tests/declared.test.ts"),
        root.join("tests/iterator.test.ts"),
        root.join("tests/mutation.test.ts"),
        root.join("tests/opaque-use.test.ts"),
        root.join("tests/shadow.test.ts"),
        root.join("tests/unconfigured.test.ts"),
        root.join("db/index.ts"),
        root.join("other/index.ts"),
    ];
    let sources = super::super::super::source_store_for_files(&files);
    let mut config = crate::config::v2::load_v2_config(&root, Some(&root.join("low.yml"))).unwrap();
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
    let source = std::fs::read_to_string(&files[0]).unwrap();
    let line = source
        .lines()
        .position(|text| text.contains("declared factory DROP"))
        .unwrap()
        + 1;
    assert!(
        findings.iter().any(|row| {
            row.file == "tests/declared.test.ts"
                && row.line == line
                && row.target.as_deref() == Some("DROP TABLE")
        }),
        "configured relative factory did not expose executed SQL: {findings:?}"
    );
    let iterator = std::fs::read_to_string(&files[1]).unwrap();
    for marker in ["factory whole DROP origin", "factory fragment DROP origin"] {
        let line = iterator
            .lines()
            .position(|text| text.contains(marker))
            .unwrap()
            + 1;
        assert!(
            findings.iter().any(|row| {
                row.file == "tests/iterator.test.ts"
                    && row.line == line
                    && row.target.as_deref() == Some("DROP TABLE")
            }),
            "{marker}: findings={findings:?}, verdicts={:?}",
            facts.sql_verdicts()
        );
    }
    let ledger = facts.sql_verdicts();
    assert!(
        ledger.iter().any(|row| {
            row.file == "tests/declared.test.ts"
                && row.status == crate::codebase::postgres::SqlVerdictStatus::Violation
                && row.residual_reasons.is_empty()
        }),
        "missing definite SQL obligation: {ledger:?}"
    );
    assert!(
        ledger.iter().any(|row| {
            row.file == "tests/declared.test.ts"
                && row.status == crate::codebase::postgres::SqlVerdictStatus::ProvenClear
                && row.residual_reasons.is_empty()
        }),
        "safe SQL at the same factory was not judged: {ledger:?}"
    );
    for file in [
        "tests/mutation.test.ts",
        "tests/opaque-use.test.ts",
        "tests/shadow.test.ts",
        "tests/unconfigured.test.ts",
    ] {
        assert!(
            !findings
                .iter()
                .any(|row| row.file == file && row.target.as_deref() == Some("DROP TABLE")),
            "unsupported factory gained a definite finding: {file}: {findings:?}"
        );
    }
    assert!(
        ledger.iter().any(|row| {
            row.file == "tests/mutation.test.ts"
                && row.status == crate::codebase::postgres::SqlVerdictStatus::Abstain
                && row.advisory
        }),
        "mutable factory receiver lost its uncertainty: {ledger:?}"
    );
    assert!(
        ledger.iter().any(|row| {
            row.file == "tests/opaque-use.test.ts"
                && row.status == crate::codebase::postgres::SqlVerdictStatus::Abstain
                && row.advisory
        }),
        "escaped finite map lost its uncertainty: {ledger:?}"
    );
}

#[test]
fn declared_factory_needs_rule_opt_in_and_selected_source() {
    let root = fixture().join("effort-factory");
    let files = [
        root.join("tests/declared.test.ts"),
        root.join("tests/unconfigured.test.ts"),
        root.join("db/index.ts"),
        root.join("other/index.ts"),
    ];
    for config_name in ["no-opt-in.yml", "excluded.yml"] {
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(config_name))).unwrap();
        let findings = check_with_files(&root, &config, &files).unwrap();
        assert!(
            !findings
                .iter()
                .any(|row| row.target.as_deref() == Some("DROP TABLE")),
            "{config_name} unexpectedly promoted a factory: {findings:?}"
        );
    }
}
