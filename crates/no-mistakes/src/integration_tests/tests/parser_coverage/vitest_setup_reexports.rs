use super::*;

#[test]
fn static_setup_reexports_resolve_commonjs_members_and_imported_defaults() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    );
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_vitest_fixture(&source, &path, &root).unwrap();
    let project = |name: &str| {
        projects
            .iter()
            .find(|project| project.policy_name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("missing project {name}"))
    };

    assert!(project("commonjs-values").vitest_setup.iter().any(|setup| {
        setup.resolved_path == Some(root.join("commonjs-values/setup/commonjs-module-named.ts"))
    }));
    let reexported = &project("imported-default-local-reexport").vitest_setup;
    assert_eq!(reexported.len(), 1);
    assert_eq!(
        reexported[0].resolved_path.as_deref(),
        Some(
            root.join("imported-default-local-reexport/setup/default-imported.ts")
                .as_path()
        )
    );
    assert_eq!(
        reexported[0].declaration_path,
        root.join("config/default-imported-setups.ts"),
        "the final static helper owns the resolved setup declaration"
    );
    let replacement = &project("commonjs-replacement").vitest_setup;
    assert_eq!(replacement.len(), 4, "{replacement:#?}");
    let resolved = replacement
        .iter()
        .filter_map(|setup| setup.resolved_path.as_ref())
        .collect::<Vec<_>>();
    assert_eq!(
        resolved,
        [
            &root.join("shared-setup/alias-barrier-retained.ts"),
            &root.join("shared-setup/module-override.ts"),
        ],
        "a module.exports replacement shadows old named values and detaches exports"
    );
    assert_eq!(
        replacement
            .iter()
            .filter(|setup| setup.specifier.is_none())
            .count(),
        2,
        "named values missing from a final object or non-object replacement stay dynamic"
    );
}
