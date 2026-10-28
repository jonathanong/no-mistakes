use super::*;

#[test]
fn low_uses_prepared_syntax_and_keeps_unproved_calls_visible() {
    let root = fixture().join("effort-low");
    let path = root.join("tests/mutation.test.ts");
    let files = vec![path.clone()];
    let source = std::fs::read_to_string(&path).unwrap();
    let lines = |marker: &str| {
        source
            .lines()
            .position(|line| line.contains(marker))
            .unwrap()
            + 1
    };
    let run = |name: &str| {
        let mut config = crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap();
        let sources = super::super::super::source_store_for_files(&files);
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
    };
    let (low, facts) = run("low.yml");
    assert!(
        facts.postgres.embedded.is_empty(),
        "low must not run max embedded extraction"
    );
    let low_models = &facts.ts[&path].sql_identifier_syntax.recovery;
    assert!(
        low_models.calls.is_empty() && low_models.functions.is_empty(),
        "direct-only test-state SQL must not lower helper models"
    );
    let (max, _) = run("max.yml");
    for (marker, origin) in [
        ("direct mutation", "direct mutation"),
        ("one local const mutation", "const sleep"),
        ("prefix CREATE", "prefix CREATE"),
        ("prefix SET", "prefix SET"),
        ("prefix sleep", "prefix sleep"),
        ("const prefix call", "const prefix DROP"),
        ("multiline direct prefix", "DROP TABLE app.multiline;"),
    ] {
        let line = lines(origin);
        assert!(
            low.iter().any(|finding| finding.line == line),
            "{marker}: {low:?}"
        );
        assert!(
            max.iter().any(|finding| finding.line == line),
            "{marker}: {max:?}"
        );
    }
    assert!(!low
        .iter()
        .any(|finding| finding.line == lines("safe direct SQL")));
    for marker in [
        "imported member receiver advisory",
        "namespace member receiver advisory",
        "awaited object member advisory",
        "quoted semicolon stays safe",
        "commented semicolon stays safe",
        "invalid prefix template",
        "mixed helper remains unproved",
        "escaped prefix stays unproved",
        "no-semicolon interpolation stays unproved",
        "mutable assignment stays unproved",
        "destructuring assignment stays unproved",
        "escaped string stays unproved",
    ] {
        assert!(
            !low.iter().any(|finding| finding.line == lines(marker)),
            "{marker}: {low:?}"
        );
    }
    for marker in [
        "imported member receiver advisory",
        "namespace member receiver advisory",
        "awaited object member advisory",
        "residual helper",
        "residual executor alias",
        "residual invalid SQL",
        "safe prefix, residual tail",
        "invalid prefix template",
        "mixed helper remains unproved",
        "escaped prefix stays unproved",
        "no-semicolon interpolation stays unproved",
        "mutable assignment stays unproved",
        "destructuring assignment stays unproved",
        "escaped string stays unproved",
    ] {
        let line = lines(marker);
        assert!(
            facts.sql_verdicts().iter().any(|row| {
                crate::codebase::ts_source::byte_offset_to_line(&source, row.span.0 as usize)
                    as usize
                    == line
                    && row.advisory
                    && row.status == crate::codebase::postgres::SqlVerdictStatus::Abstain
            }),
            "{marker}: {:?}",
            facts.sql_verdicts()
        );
    }
    assert!(!facts.sql_verdicts().iter().any(|row| {
        crate::codebase::ts_source::byte_offset_to_line(&source, row.span.0 as usize) as usize
            == lines("unrelated local member stays outside scope")
    }));
    let (reported, reported_facts) = run("report.yml");
    assert!(reported
        .iter()
        .any(|finding| finding.line == lines("prefix CREATE")
            && finding.target.as_deref() == Some("CREATE TABLE")));
    assert!(reported.iter().any(
        |finding| finding.line == lines("safe prefix, residual tail")
            && finding.target.as_deref() == Some("unanalyzable-sql")
    ));
    assert!(!reported
        .iter()
        .any(|finding| finding.line == lines("prefix CREATE")
            && finding.target.as_deref() == Some("unanalyzable-sql")));
    assert!(!reported_facts.sql_verdicts().iter().any(|row| row.advisory));
    let (ignored, ignored_facts) = run("ignore.yml");
    assert!(ignored
        .iter()
        .any(|finding| finding.line == lines("prefix CREATE")
            && finding.target.as_deref() == Some("CREATE TABLE")));
    assert!(!ignored.iter().any(
        |finding| finding.line == lines("safe prefix, residual tail")
            && finding.target.as_deref() == Some("unanalyzable-sql")
    ));
    assert!(!ignored_facts.sql_verdicts().iter().any(|row| row.advisory));
}

#[test]
fn mixed_low_sql_preserves_other_helper_recovery_without_proving_test_state() {
    let root = fixture().join("effort-low");
    let path = root.join("tests/mutation.test.ts");
    let files = vec![path.clone()];
    let sources = super::super::super::source_store_for_files(&files);
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("mixed.yml"))).unwrap();
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
        sources,
        &config,
        &[RULE_ID, "postgres-no-offset"],
    )
    .unwrap();
    let syntax = &facts.ts[&path].sql_identifier_syntax;
    assert!(!syntax.recovery.calls.is_empty() && !syntax.recovery.functions.is_empty());
    let source = std::fs::read_to_string(&path).unwrap();
    let mixed_line = source
        .lines()
        .position(|line| line.contains("mixed helper remains unproved"))
        .unwrap() as u32
        + 1;
    let member_line = source
        .lines()
        .position(|line| line.contains("imported member receiver advisory"))
        .unwrap() as u32
        + 1;
    for key in syntax.application_keys() {
        let member = syntax
            .calls_for(&key)
            .unwrap()
            .iter()
            .find(|call| call.line == member_line)
            .unwrap();
        assert!(matches!(
            member.input,
            crate::codebase::postgres::sql_identifier_syntax::Input::Unknown
        ));
        assert!(!syntax
            .recovery
            .sinks
            .get(&key.profile)
            .is_some_and(|sinks| sinks.contains(&member.span)));
        let call = syntax
            .calls_for(&key)
            .unwrap()
            .iter()
            .find(|call| call.line == mixed_line)
            .unwrap();
        if key.rule == RULE_ID {
            assert!(matches!(
                call.input,
                crate::codebase::postgres::sql_identifier_syntax::Input::Unknown
            ));
        } else if key.rule == "postgres-no-offset" {
            assert!(matches!(
                call.input,
                crate::codebase::postgres::sql_identifier_syntax::Input::Recovered(_)
            ));
        }
    }
}

#[test]
fn standalone_low_entrypoint_matches_prepared_rule_application() {
    let root = fixture().join("effort-low");
    let path = root.join("tests/mutation.test.ts");
    let files = vec![path.clone()];
    let unprepared =
        crate::config::v2::load_v2_config(&root, Some(&root.join("report.yml"))).unwrap();
    let sources = super::super::super::source_store_for_files(&files);
    let mut prepared = unprepared.clone();
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        &root,
        &mut prepared,
        &files,
        &sources,
    )
    .unwrap();
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        Arc::clone(&sources),
        &prepared,
        &[RULE_ID],
    )
    .unwrap();
    let expected =
        check_with_files_sources_and_facts(&root, &prepared, &files, &sources, &facts).unwrap();
    let actual = check_with_files(&root, &unprepared, &files).unwrap();
    assert_eq!(actual, expected);
    let source = std::fs::read_to_string(&path).unwrap();
    let helper_line = source
        .lines()
        .position(|line| line.contains("mixed helper remains unproved"))
        .unwrap()
        + 1;
    assert!(
        actual.iter().any(|finding| finding.line == helper_line
            && finding.target.as_deref() == Some("unanalyzable-sql")),
        "{actual:?}"
    );
}
