use super::*;

#[test]
fn closed_consumer_candidates_keep_annotation_proofs_separate_from_complete_sql() {
    let base = crate::test_support::rule_fixture_root("postgres-require-query-annotation");
    for (directory, config, file) in [
        (
            "low-heads",
            "finite-conditional-consumer-low.yml",
            "finite-conditional-consumer.mts",
        ),
        (
            "closed-consumer-candidates",
            "closed-consumer-candidates-low.yml",
            "privacy.mts",
        ),
    ] {
        let root = base.join(directory);
        let config = base.parent().unwrap().join("configs").join(config);
        let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
        let report = {
            let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
            crate::test_support::without_hygiene_advisories(
                run_all_with_sql_verdicts(root.clone(), Some(config), None, false, false, false)
                    .unwrap(),
            )
        };
        assert!(report.rules.is_empty(), "{:?}", report.rules);
        let source = std::fs::read_to_string(root.join(file)).unwrap();
        let expected = marked_lines(&source, "// low: false");
        assert!(
            !expected.is_empty(),
            "unsafe controls must remain observable"
        );
        assert_eq!(call_lines(&report.advisories, file), expected);
        if directory == "closed-consumer-candidates" {
            assert!(call_lines(&report.advisories, "retained-relations.mts").is_empty());
        }
        assert_no_annotation_evaluation(&observer);
    }
}
