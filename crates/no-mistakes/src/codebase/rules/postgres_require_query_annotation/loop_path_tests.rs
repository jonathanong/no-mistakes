use super::*;

#[test]
fn conditional_paths_preserve_only_live_backedge_witnesses() {
    let root = super::tests::fixture("loop-path-witness");
    let file = root.join("src/query.mts");
    let source = std::fs::read_to_string(&file).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("// missing:").then_some(index + 1))
        .collect::<Vec<_>>();
    let low_expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            (line.contains("read(q)") && !line.contains("low-unknown")).then_some(index + 1)
        })
        .collect::<Vec<_>>();
    for (effort, expected_lines) in [
        (crate::config::v2::Effort::Low, low_expected),
        (crate::config::v2::Effort::Max, expected.clone()),
    ] {
        let mut config = super::tests::config_with_options(
            "importSpecifier: '@example/db'\nexecutorNames: [read]\nunanalyzableSql: ignore",
        );
        config.effort = Some(effort);
        let files = [file.clone()];
        let sources = super::super::source_store_for_files(&files);
        super::super::analysis_policy::prepare_rule_analysis(&root, &mut config, &files, &sources)
            .unwrap();
        assert_eq!(
            config
                .effective_rule_analysis(&config.rules[0])
                .unwrap()
                .effort,
            effort
        );
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
            super::super::prepare_sql_rule_facts(&root, &config, &files, &sources, RULE_ID).unwrap()
        });
        let roots = observer
            .snapshot()
            .work
            .get("postgres.annotation.roots")
            .copied()
            .unwrap_or(0);
        assert_eq!(roots == 0, effort == crate::config::v2::Effort::Low);
        let findings =
            check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
        let lines = findings
            .iter()
            .filter(|finding| finding.message.contains("query execution must start"))
            .map(|finding| finding.line)
            .collect::<Vec<_>>();
        assert_eq!(lines, expected_lines, "{effort:?}: {findings:?}");
    }
}

#[test]
fn malformed_loop_and_source_false_positive_do_not_build_a_binding_index() {
    for (name, expected_visits, expect_error) in [("malformed", 0, true), ("format", 0, false)] {
        let root = super::tests::fixture("loop-carried-reassignment");
        let files = [root.join(format!("src/{name}.mts"))];
        let config = super::tests::config_with_options(
            "importSpecifier: '@example/db'\nexecutorNames: [read]",
        );
        let sources = super::super::source_store_for_files(&files);
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
            super::super::prepare_sql_rule_facts(&root, &config, &files, &sources, RULE_ID).unwrap()
        });
        let work = observer.snapshot().work;
        assert_eq!(
            work.get("postgres.annotation.head_syntax_visits")
                .copied()
                .unwrap_or(0),
            expected_visits
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
        assert_eq!(
            facts.ts.values().any(|file| file.parse_error.is_some()),
            expect_error
        );
        if name == "format" {
            let findings = check_with_files(&root, &config, &files).unwrap();
            assert_eq!(findings.len(), 1, "{findings:?}");
            assert!(findings[0].message.contains("query execution must start"));
        }
    }
}

#[test]
fn called_condition_writes_invalidate_only_the_same_binding() {
    let root = super::tests::fixture("loop-condition-writes");
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    for effort in ["low", "max"] {
        let expected = source
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                (if effort == "low" {
                    line.contains("read(q)")
                } else {
                    line.contains("// missing:")
                })
                .then_some(index + 1)
            })
            .collect::<Vec<_>>();
        let mut config = crate::config::v2::load_v2_config(
            &root,
            Some(&root.join(format!(".no-mistakes-{effort}.yml"))),
        )
        .unwrap();
        let files = [root.join("src/query.mts")];
        let sources = super::super::source_store_for_files(&files);
        super::super::analysis_policy::prepare_rule_analysis(&root, &mut config, &files, &sources)
            .unwrap();
        assert_eq!(
            config
                .effective_rule_analysis(&config.rules[0])
                .unwrap()
                .effort,
            if effort == "low" {
                crate::config::v2::Effort::Low
            } else {
                crate::config::v2::Effort::Max
            }
        );
        let findings = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
        let lines = findings
            .iter()
            .filter(|finding| {
                finding.rule == RULE_ID && finding.message.contains("query execution must start")
            })
            .map(|finding| finding.line)
            .collect::<Vec<_>>();
        assert_eq!(lines, expected, "{effort}: {findings:#?}");
    }
}
