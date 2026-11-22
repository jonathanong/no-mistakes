use super::*;

#[test]
fn staged_and_streaming_facts_count_supplemental_vitest_configs_once() {
    use crate::config::v2::schema::{RuleDef, RuleScope};

    let root = crate::test_support::rule_fixture_root("vitest-default-excludes")
        .join("supplemental")
        .canonicalize()
        .unwrap();
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    config.rules.push(RuleDef {
        rule: "postgres-require-query-annotation".into(),
        scope: Some(RuleScope::Repository),
        include: vec!["absent-query.ts".into()],
        options: serde_yaml::from_str(
            "effort: max\nimportSpecifier: '@app/db'\nexecutorNames: [query]",
        )
        .unwrap(),
        ..Default::default()
    });
    let vitest_paths =
        crate::codebase::rules::vitest_default_excludes::required_files(&root, &config).unwrap();
    let vitest = root.join("skipped/vitest.config.ts");
    let missing = root.join("missing.config.ts");
    assert_eq!(vitest_paths.len(), 2);

    for staged in [true, false] {
        let observer = InvocationObserver::new(true);
        let session = AnalysisSession::new(Some(Arc::clone(&observer)));
        let (_, mut plan, _, sources) = prepare_config(&session, &root, config.clone());
        plan.vitest_default_exclude_paths = Arc::clone(&vitest_paths);
        if !staged {
            plan.query_annotation_catalog = None;
        }
        let files = vec![root.join("consumer.ts"), root.join("shared.ts")];
        let facts = crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
            check_facts::collect_prepared_check_facts(
                &session,
                &root,
                (files.clone(), Vec::new()),
                Vec::new(),
                plan,
                None,
                sources,
            )
        });
        assert_eq!(facts.files, files);
        assert!(facts.graph_files.is_empty());
        assert_eq!(facts.stats.files_discovered, 4);
        assert_eq!(facts.stats.files_parsed, 3);
        assert_eq!(facts.stats.parse_errors, 1);
        assert_eq!(facts.ts[&vitest].vitest_default_excludes.len(), 1);
        assert!(facts.ts[&missing].parse_error.is_some());
        let work = observer.snapshot().work;
        assert_eq!(
            work.get("postgres.annotation.syntax_collections")
                .copied()
                .unwrap_or(0),
            if staged { 0 } else { 2 }
        );
        assert_eq!(session.work_snapshot().parse_attempts[&vitest], 1);
    }
}
