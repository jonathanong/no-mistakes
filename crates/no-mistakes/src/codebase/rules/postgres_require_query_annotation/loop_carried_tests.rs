// Output cases moved to cli_postgres_snapshot_require_query_annotation.
use super::*;
use crate::codebase::postgres::SqlVerdictStatus;

#[test]
fn loop_carried_literal_missing_heads_reach_both_efforts() {
    for effort in [
        crate::config::v2::Effort::Low,
        crate::config::v2::Effort::Max,
    ] {
        for (mode, name) in ["ignore", "advise", "report"]
            .into_iter()
            .flat_map(|mode| ["query", "while-only"].map(|name| (mode, name)))
        {
            let root = super::tests::fixture("loop-carried-reassignment");
            let file = root.join(format!("src/{name}.mts"));
            let source = std::fs::read_to_string(&file).unwrap();
            let expected = source
                .lines()
                .enumerate()
                .filter_map(|(index, line)| line.contains("// missing:").then_some(index + 1))
                .collect::<Vec<_>>();
            let mut config = super::tests::config_with_options(&format!(
                "importSpecifier: '@example/db'\nexecutorNames: [read, query]\nexecutorTypeNames: [Database]\nunanalyzableSql: {mode}"
            ));
            config.effort = Some(effort);
            let files = [file];
            let sources = super::super::source_store_for_files(&files);
            super::super::analysis_policy::prepare_rule_analysis(
                &root,
                &mut config,
                &files,
                &sources,
            )
            .unwrap();
            assert_eq!(
                config
                    .effective_rule_analysis(&config.rules[0])
                    .unwrap()
                    .effort,
                effort
            );
            let observer = crate::diagnostics::InvocationObserver::new(true);
            let mut facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
                super::super::prepare_sql_rule_facts(&root, &config, &files, &sources, RULE_ID)
                    .unwrap()
            });
            let work = observer.snapshot().work;
            let roots = work.get("postgres.annotation.roots").copied().unwrap_or(0);
            assert_eq!(roots == 0, effort == crate::config::v2::Effort::Low);
            assert_eq!(work.get("postgres.annotation.head_syntax_visits"), Some(&1));
            assert_eq!(
                work.get("postgres.annotation.head_binding_indexes"),
                Some(&1)
            );
            facts.enable_sql_verdicts();
            let _findings =
                check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts)
                    .unwrap();
            for line in &expected {
                let verdicts = facts.sql_verdicts();
                let verdict = verdicts
                    .iter()
                    .find(|verdict| {
                        crate::codebase::ts_source::byte_offset_to_line(
                            &source,
                            verdict.span.0 as usize,
                        ) as usize
                            == *line
                    })
                    .unwrap();
                assert_eq!(verdict.status, SqlVerdictStatus::Violation);
                assert!(!verdict.advisory);
            }
        }
    }
}

#[test]
fn max_without_loop_keeps_the_head_visitor_and_binding_index_cold() {
    use crate::diagnostics::{with_observer, InvocationObserver};
    let root = super::tests::fixture("fail");
    let files = [root.join("src/query.ts")];
    let config = super::tests::config_with_options("importSpecifier: '@example/db'");
    let sources = super::super::source_store_for_files(&files);
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        super::super::prepare_sql_rule_facts(&root, &config, &files, &sources, RULE_ID).unwrap()
    });
    let work = observer.snapshot().work;
    assert_eq!(
        work.get("postgres.annotation.head_syntax_visits")
            .copied()
            .unwrap_or(0),
        0
    );
    assert_eq!(
        work.get("postgres.annotation.head_binding_indexes")
            .copied()
            .unwrap_or(0),
        0
    );
    assert!(facts
        .ts
        .values()
        .all(|file| file.query_annotation_heads.is_none()));
}

#[test]
fn loop_carried_findings_keep_executor_suppression_locations() {
    let root = super::tests::fixture("loop-carried-reassignment");
    let files = [
        root.join("src/suppressed.mts"),
        root.join("src/file-suppressed.mts"),
    ];
    let config = super::tests::config_with_options(
        "importSpecifier: '@example/db'\nexecutorNames: [read]\nunanalyzableSql: ignore",
    );
    let sources = super::super::source_store_for_files(&files);
    let facts =
        super::super::prepare_sql_rule_facts(&root, &config, &files, &sources, RULE_ID).unwrap();
    let mut findings =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    assert_eq!(findings.len(), 3, "{findings:?}");
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:?}");
}
