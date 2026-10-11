use super::*;

#[test]
fn callback_setup_contexts_follow_syntax_roots_and_project_boundaries() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-runner-executions/callback-syntax"),
    );
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
    for (file, config_file, expected, target, setups) in [
        (
            ".no-mistakes.yml",
            "vitest.config.mts",
            "tests/multi.test.mts",
            "src/multi.mts",
            vec!["multi.setup.mts"],
        ),
        (
            ".no-mistakes-root.yml",
            "root.config.mts",
            "web/tests/genuine.test.mts",
            "web/src/root-target.mts",
            vec!["web/setup.mts"],
        ),
        (
            ".no-mistakes-comments.yml",
            "comment.config.mts",
            "tests/comments.test.mts",
            "src/comment.mts",
            vec![],
        ),
        (
            ".no-mistakes-opaque.yml",
            "opaque.config.mts",
            "tests/opaque.test.mts",
            "src/opaque.mts",
            vec![],
        ),
    ] {
        let path = root.join(file);
        let config = crate::config::v2::load_v2_config(&root, Some(&path)).unwrap();
        crate::ast::begin_parse_count(&root);
        let standalone = check(&root, &config, None).unwrap();
        let counts = crate::ast::finish_parse_count(&root);
        assert_eq!(counts.get(&root.join(config_file)), Some(&1));
        for setup in setups {
            assert_eq!(counts.get(&root.join(setup)), Some(&1));
        }
        for findings in [
            standalone,
            check_with_facts(&root, &config, None, &facts).unwrap(),
            crate::codebase::rules::run_check_with_facts(&root, Some(&path), None, &facts).unwrap(),
        ] {
            assert_eq!(findings.len(), 1, "{file}: {findings:?}");
            assert_eq!(findings[0].file, expected);
            assert_eq!(findings[0].target.as_deref(), Some(target));
        }
    }
}
