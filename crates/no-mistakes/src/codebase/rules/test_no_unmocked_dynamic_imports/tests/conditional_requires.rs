use super::*;

#[test]
fn skipped_commonjs_helpers_do_not_cover_dynamic_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-runner-executions/conditional-requires"),
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
    crate::ast::begin_parse_count(&root);
    let standalone = check(&root, &config, None).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    for file in [
        "vitest.config.mts",
        "setup.cjs",
        "helpers/live.cjs",
        "tests/conditionals.test.cjs",
        "tests/do-while-eager.test.cjs",
        "tests/annex-b.test.cjs",
    ] {
        assert_eq!(counts.get(&root.join(file)), Some(&1), "{file}: {counts:?}");
    }
    for findings in [
        standalone,
        check_with_facts(&root, &config, None, &facts).unwrap(),
        crate::codebase::rules::run_check_with_facts(&root, None, None, &facts).unwrap(),
    ] {
        let actual: Vec<_> = findings
            .iter()
            .map(|finding| (finding.file.as_str(), finding.target.as_deref()))
            .collect();
        assert_eq!(
            actual,
            [
                ("tests/conditionals.test.cjs", Some("src/target.cjs")),
                ("tests/setup-skipped.test.cjs", Some("src/setup-target.cjs")),
            ]
        );
    }
}
