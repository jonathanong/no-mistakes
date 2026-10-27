// Output cases moved to cli_postgres_snapshot_require_query_annotation.
use super::*;

#[test]
fn conditional_paths_preserve_only_live_backedge_witnesses() {
    let root = super::tests::fixture("loop-path-witness");
    let file = root.join("src/query.mts");
    for effort in [
        crate::config::v2::Effort::Low,
        crate::config::v2::Effort::Max,
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
        let _facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
            super::super::prepare_sql_rule_facts(&root, &config, &files, &sources, RULE_ID).unwrap()
        });
        let roots = observer
            .snapshot()
            .work
            .get("postgres.annotation.roots")
            .copied()
            .unwrap_or(0);
        assert_eq!(roots == 0, effort == crate::config::v2::Effort::Low);
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
    }
}

#[test]
fn called_condition_writes_invalidate_only_the_same_binding() {
    let root = super::tests::fixture("loop-condition-writes");
    for effort in ["low", "max"] {
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
    }
}
