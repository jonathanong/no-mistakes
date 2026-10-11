use super::*;

#[test]
fn static_setup_fallback_applies_only_when_no_project_is_recoverable() {
    for (kind, expected, target) in [
        ("callback", "tests/genuine.test.mts", "src/genuine.mts"),
        (
            "unselected-project",
            "tests/covered.test.mts",
            "src/target.mts",
        ),
    ] {
        let root = crate::codebase::ts_resolver::normalize_path(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-runner-executions")
                .join(kind),
        );
        let config = crate::config::v2::load_v2_config(&root, None).unwrap();
        let facts = crate::codebase::check_facts::collect_check_facts(
            &root,
            crate::codebase::ts_source::discover_files(&root, &[]),
            crate::codebase::check_facts::CheckFactPlan {
                imports: true,
                dynamic_imports: true,
                source: true,
                ..Default::default()
            },
        );
        // These dedicated fixtures include setup parsing on fact-collection workers.
        crate::ast::begin_parse_count(&root);
        let standalone = check(&root, &config, None).unwrap();
        let counts = crate::ast::finish_parse_count(&root);
        assert_eq!(counts.get(&root.join("vitest.config.mts")), Some(&1));
        if kind == "callback" {
            assert_eq!(counts.get(&root.join("setup.mts")), Some(&1));
        }
        for findings in [
            standalone,
            check_with_facts(&root, &config, None, &facts).unwrap(),
            crate::codebase::rules::run_check_with_facts(&root, None, None, &facts).unwrap(),
        ] {
            assert_eq!(findings.len(), 1, "{kind}: {findings:?}");
            assert_eq!(findings[0].file, expected);
            assert_eq!(findings[0].target.as_deref(), Some(target));
        }
    }
}
