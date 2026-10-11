use super::*;

#[test]
fn callback_aliases_and_spreads_preserve_actual_setup_coverage() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-runner-executions/callback-aliases"),
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
    for (name, expected, setup) in [
        ("mutable", Some("tests/mutable.test.mts"), None),
        (
            "named",
            Some("tests/named-genuine.test.mts"),
            Some("named.setup.mts"),
        ),
        ("spread", Some("tests/spread.test.mts"), None),
        ("spread-root", Some("tests/spread-root.test.mts"), None),
        (
            "aliased-spread",
            Some("tests/aliased-spread.test.mts"),
            None,
        ),
        ("mutable-test", Some("tests/mutable-test.test.mts"), None),
        ("known", None, Some("known.setup.mts")),
    ] {
        let path = root.join(format!(".no-mistakes-{name}.yml"));
        let config = crate::config::v2::load_v2_config(&root, Some(&path)).unwrap();
        crate::ast::begin_parse_count(&root);
        let standalone = check(&root, &config, None).unwrap();
        let counts = crate::ast::finish_parse_count(&root);
        assert_eq!(
            counts.get(&root.join(format!("{name}.config.mts"))),
            Some(&1)
        );
        if let Some(setup) = setup {
            assert_eq!(counts.get(&root.join(setup)), Some(&1));
        }
        for findings in [
            standalone,
            check_with_facts(&root, &config, None, &facts).unwrap(),
            crate::codebase::rules::run_check_with_facts(&root, Some(&path), None, &facts).unwrap(),
        ] {
            let files: Vec<_> = findings
                .iter()
                .map(|finding| finding.file.as_str())
                .collect();
            assert_eq!(files, expected.into_iter().collect::<Vec<_>>(), "{name}");
        }
    }
}
