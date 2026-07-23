use super::*;

#[test]
fn imported_default_exclusion_reexports_parse_each_helper_once_per_phase() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-project-entries");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let visible = crate::codebase::ts_source::discover_visible_paths(&root)
        .into_iter()
        .collect();
    let tsconfig = test_support::tsconfig_without_config(&root);

    crate::ast::begin_parse_count(&root);
    let projects =
        test_support::parse_vitest_from_visible(&source, &path, &root, &root, &tsconfig, &visible)
            .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert!(projects
        .iter()
        .all(|project| project.policy_name.as_deref() != Some("negated-default-imported")));
    assert_eq!(
        counts.get(&root.join("projects/default-imported-exclusions-values.ts")),
        Some(&3),
        "each required exclusion phase follows the default re-export once"
    );
}
