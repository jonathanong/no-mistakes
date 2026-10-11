use super::*;

#[test]
fn eager_requires_preserve_control_flow_and_recovered_facts() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-runner-executions/require-fidelity"),
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
    let recovered = &facts.ts[&root.join("helpers/recovered.mts")];
    assert!(recovered.parse_error.is_some());
    assert_eq!(recovered.parse_error, recovered.ts.parse_error);
    assert_eq!(recovered.ts.eager_require_specifiers, ["./nested.cjs"]);
    crate::ast::begin_parse_count(&root);
    let standalone = check(&root, &config, None).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    for file in [
        "vitest.config.mts",
        "setup.cjs",
        "helpers/recovered.mts",
        "helpers/nested.cjs",
        "helpers/switch.cjs",
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
                ("tests/loop.test.cjs", Some("src/loop.cjs")),
                ("tests/zero.test.cjs", Some("src/loop.cjs")),
            ]
        );
    }
}
