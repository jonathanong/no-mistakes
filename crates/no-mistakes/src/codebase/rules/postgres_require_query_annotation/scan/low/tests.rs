use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::codebase::rules::postgres_require_query_annotation::{compile_options, Options};

fn low_fixture(
    mode: crate::codebase::rules::postgres_require_query_annotation::UnanalyzableSql,
) -> (
    Vec<crate::codebase::rules::RuleFinding>,
    Vec<(usize, crate::codebase::postgres::SqlObligationVerdict)>,
) {
    use crate::codebase::postgres::SqlVerdictStatus;
    use crate::codebase::ts_source::byte_offset_to_line;

    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let files = vec![root.join("low-missing.mts")];
    let opts = compile_options(&Options {
        import_specifier: "@bench/db".into(),
        executor_names: vec!["read".into()],
        unanalyzable_sql: mode,
        ..Default::default()
    })
    .unwrap();
    let mut facts = collect_check_facts(
        root,
        files.clone(),
        CheckFactPlan {
            query_annotation_heads: true,
            embedded_sql_options: vec![opts.embedded.clone()],
            ..Default::default()
        },
    );
    facts.enable_sql_verdicts();
    let findings = super::scan(root, &opts, &files, &facts, None)
        .unwrap()
        .finish();
    let source = std::fs::read_to_string(&files[0]).unwrap();
    let verdicts = facts.sql_verdicts();
    let verdicts = verdicts
        .iter()
        .map(|verdict| {
            (
                byte_offset_to_line(&source, verdict.span.0 as usize) as usize,
                verdict.clone(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(verdicts.len(), 43);
    assert_eq!(
        verdicts
            .iter()
            .filter_map(|(line, verdict)| {
                (verdict.status == SqlVerdictStatus::Violation).then_some(*line)
            })
            .collect::<Vec<_>>(),
        [3, 4, 5, 7, 8, 11, 19, 21, 27, 29, 33, 34, 35, 36, 37, 38, 39, 42, 43, 44, 47]
    );
    (findings, verdicts)
}

#[test]
fn direct_literal_missing_heads_violate_under_every_policy_but_holes_stay_unresolved() {
    use crate::codebase::postgres::SqlVerdictStatus;
    use crate::codebase::rules::postgres_require_query_annotation::UnanalyzableSql;

    for (mode, expected_advisory) in [
        (UnanalyzableSql::Report, false),
        (UnanalyzableSql::Advise, true),
        (UnanalyzableSql::Ignore, false),
    ] {
        let (findings, verdicts) = low_fixture(mode);
        let finding_lines = findings
            .into_iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>();
        let expected = if mode == UnanalyzableSql::Report {
            [
                3, 4, 5, 7, 8, 11, 12, 13, 14, 15, 16, 17, 18, 19, 21, 22, 23, 24, 25, 27, 29, 32,
                33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 47,
            ]
            .to_vec()
        } else {
            vec![
                3, 4, 5, 7, 8, 11, 19, 21, 27, 29, 33, 34, 35, 36, 37, 38, 39, 42, 43, 44, 47,
            ]
        };
        assert_eq!(finding_lines, expected);
        for (line, verdict) in &verdicts {
            if [
                3, 4, 5, 7, 8, 11, 19, 21, 27, 29, 33, 34, 35, 36, 37, 38, 39, 42, 43, 44, 47,
            ]
            .contains(line)
            {
                assert_eq!(verdict.status, SqlVerdictStatus::Violation, "line {line}");
                assert!(!verdict.advisory, "line {line}");
            } else if [9, 10, 26, 45, 46, 48, 49, 50].contains(line) {
                assert_eq!(verdict.status, SqlVerdictStatus::ProvenClear, "line {line}");
                assert!(!verdict.advisory, "line {line}");
            } else {
                assert_eq!(verdict.status, SqlVerdictStatus::Abstain, "line {line}");
                assert_eq!(verdict.advisory, expected_advisory, "line {line}");
            }
        }
    }
}

#[test]
fn borrowed_observer_counts_calls_on_a_worker_without_diagnostic_tls() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let files = vec![root.join("direct-only.mts")];
    let opts = compile_options(&Options {
        import_specifier: "@bench/db".into(),
        executor_names: vec!["read".into()],
        ..Default::default()
    })
    .unwrap();
    let facts = collect_check_facts(
        root,
        files.clone(),
        CheckFactPlan {
            query_annotation_heads: true,
            embedded_sql_options: vec![opts.embedded.clone()],
            ..Default::default()
        },
    );
    let expected_calls = facts.ts.get(&files[0]).unwrap().embedded_sql[0]
        .1
        .calls
        .len();
    assert_eq!(expected_calls, 2);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let (without, with) = std::thread::scope(|scope| {
        scope
            .spawn(|| {
                // A stolen worker has no caller TLS. Both scans must still
                // agree, and the explicitly borrowed observer must see work.
                assert!(crate::diagnostics::current().is_none());
                (
                    super::scan(root, &opts, &files, &facts, None).unwrap(),
                    super::scan(root, &opts, &files, &facts, Some(&observer)).unwrap(),
                )
            })
            .join()
            .unwrap()
    });
    assert_eq!(without.finish(), with.finish());
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("postgres.annotation.low.calls"),
        Some(&(expected_calls as u64))
    );
}

#[test]
fn default_sql_template_tag_missing_heads_violate_at_low_effort() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let files = vec![root.join("default-tags.mts")];
    let opts = compile_options(&Options {
        import_specifier: "@bench/db".into(),
        executor_names: vec!["read".into()],
        ..Default::default()
    })
    .unwrap();
    let facts = collect_check_facts(
        root,
        files.clone(),
        CheckFactPlan {
            query_annotation_heads: true,
            embedded_sql_options: vec![opts.embedded.clone()],
            ..Default::default()
        },
    );
    let findings = super::scan(root, &opts, &files, &facts, None)
        .unwrap()
        .finish();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        vec![7, 9]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.message.contains("query execution must start")));
}
