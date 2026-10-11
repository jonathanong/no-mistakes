use super::*;

#[test]
fn eager_switch_bigint_and_recovered_helpers_match_standalone_prepared_and_aggregate() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-runner-executions/eager-switch-bigint",
        ),
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
    let recovered_file = &facts.ts[&root.join("helpers/recovered.ts")];
    let recovered = &recovered_file.ts;
    assert!(recovered.parse_error.is_some());
    assert_eq!(recovered.eager_require_specifiers, ["./mock.cjs"]);
    assert!(recovered
        .imports
        .iter()
        .any(|import| import.specifier == "./mock.cjs"));
    assert!(Arc::ptr_eq(
        recovered_file.source.as_ref().unwrap(),
        recovered.source.as_ref().unwrap(),
    ));
    crate::ast::begin_parse_count(&root);
    let standalone = check(&root, &config, None).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    for file in [
        "vitest.config.mts",
        "setup.cjs",
        "helpers/mock.cjs",
        "helpers/recovered.ts",
        "tests/switch-first.test.cjs",
        "tests/bigint-truthy.test.cjs",
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
                ("tests/bigint-if-zero.test.cjs", Some("src/target.cjs")),
                ("tests/bigint-zero.test.cjs", Some("src/target.cjs")),
                ("tests/switch-later.test.cjs", Some("src/target.cjs")),
            ]
        );
    }
}
