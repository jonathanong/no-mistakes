use super::*;

#[test]
fn vitest_nested_test_setup_fields_dominate_outer_fields_in_either_order() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-config/vitest-nested-test-setups"),
    );
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_vitest_fixture(&source, &path, &root).unwrap();

    for name in [
        "outer-first",
        "outer-last",
        "local-nested-spread",
        "imported-nested-spread",
    ] {
        let matching = projects
            .iter()
            .filter(|project| project.policy_name.as_deref() == Some(name))
            .collect::<Vec<_>>();
        assert_eq!(
            matching.len(),
            if name.contains("spread") { 2 } else { 1 },
            "{name}"
        );
        assert!(
            matching
                .iter()
                .all(|project| project.vitest_setup.len() == 2),
            "{name}"
        );
        assert!(
            matching
                .iter()
                .flat_map(|project| &project.vitest_setup)
                .all(|dependency| {
                    dependency
                        .specifier
                        .as_deref()
                        .is_some_and(|specifier| specifier.contains("inner"))
                }),
            "{name}: {matching:#?}"
        );
    }
}
