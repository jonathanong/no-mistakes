use super::*;

#[test]
fn vitest_inline_setup_inheritance_requires_extends_true() {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/test-config/vitest-extends");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_vitest_fixture(&source, &path, &root).unwrap();

    for name in ["default", "false", "spread-false-last"] {
        assert!(projects
            .iter()
            .find(|project| project.policy_name.as_deref() == Some(name))
            .unwrap()
            .vitest_setup
            .is_empty());
    }
    for name in ["true", "spread-true-last"] {
        // `parse_options` always checks static config extends first; boolean
        // `true` must remain available for the later root-setup merge.
        let inherited = projects
            .iter()
            .find(|project| project.policy_name.as_deref() == Some(name))
            .unwrap();
        assert_eq!(inherited.vitest_setup.len(), 2, "{name}");
        assert_eq!(
            inherited
                .vitest_setup
                .iter()
                .map(|setup| setup.field.as_str())
                .collect::<Vec<_>>(),
            vec!["setupFiles", "globalSetup"],
            "{name}",
        );
    }
    let unresolved = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("nonboolean"))
        .unwrap();
    assert_eq!(unresolved.vitest_setup.len(), 1);
    assert_eq!(
        unresolved.vitest_setup[0]
            .unresolved_config_extends
            .as_deref(),
        Some("not-boolean")
    );
    let merged = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("merged-setups"))
        .unwrap();
    assert_eq!(
        merged
            .vitest_setup
            .iter()
            .filter(|setup| setup.field.as_str() == "setupFiles")
            .map(|setup| setup.specifier.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["./root-setup.ts", "./project-setup.ts"],
    );
    assert_merged_provenance(merged, "./root-setup.ts");
    assert_eq!(
        merged
            .vitest_setup
            .iter()
            .filter(|setup| setup.field.as_str() == "globalSetup")
            .map(|setup| setup.specifier.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["./root-global.ts", "./project-global.ts"],
    );
    assert_merged_provenance(merged, "./root-global.ts");
    let standalone = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("standalone"))
        .unwrap();
    assert_eq!(standalone.vitest_setup.len(), 1);
    assert_eq!(
        standalone.vitest_setup[0].specifier.as_deref(),
        Some("./standalone-setup.ts")
    );
}

#[test]
fn vitest_inline_setup_inheritance_resolves_static_config_extends() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-extends-config");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_vitest_fixture(&source, &path, &root).unwrap();
    let project = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("extended"))
        .unwrap();

    assert!(project
        .vitest_setup
        .iter()
        .any(|setup| setup.config_extends_provenance));
    assert_eq!(
        project
            .vitest_setup
            .iter()
            .filter(|setup| !setup.config_extends_provenance)
            .map(|setup| (setup.field.as_str(), setup.specifier.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            ("setupFiles", Some("./base-setup.ts")),
            ("setupFiles", Some("./local-setup.ts")),
            ("globalSetup", Some("./base-global.ts")),
        ]
    );
    assert!(project.vitest_setup.iter().all(|setup| {
        setup
            .trigger_paths
            .iter()
            .any(|path| path.ends_with("vite.config.js"))
            || setup.specifier.as_deref() == Some("./local-setup.ts")
    }));

    let cycle = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("cycle"))
        .unwrap();
    assert_eq!(
        cycle
            .vitest_setup
            .iter()
            .filter(|setup| !setup.config_extends_provenance)
            .map(|setup| setup.specifier.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("./cycle-setup.ts"), Some("./cycle-global.ts")]
    );

    let unresolved = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("unresolved"))
        .unwrap();
    assert_eq!(unresolved.vitest_setup.len(), 1);
    assert_eq!(
        unresolved.vitest_setup[0]
            .unresolved_config_extends
            .as_deref(),
        Some("./missing-vite.config.js")
    );

    let unsupported = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("unsupported"))
        .unwrap();
    assert!(unsupported
        .vitest_setup
        .iter()
        .any(|setup| { setup.unresolved_config_extends.as_deref() == Some("./vite-factory.js") }));
    assert!(unsupported
        .vitest_setup
        .iter()
        .any(|setup| setup.config_extends_provenance));

    let scope = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("scope-inherited"))
        .unwrap();
    assert_eq!(scope.scope.as_deref(), Some("scope-inherited"));
    assert_eq!(scope.include, ["scope-inherited/owned/**/*.spec.ts"]);
    assert_eq!(
        scope.exclude,
        [
            "scope-inherited/inherited-ignore/**",
            "scope-inherited/local-ignore/**",
        ]
    );
    assert!(scope
        .vitest_setup
        .iter()
        .any(|setup| setup.specifier.as_deref() == Some("../scope-setup.ts")));

    let cross = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("cross-inherited"))
        .unwrap();
    assert_eq!(
        cross.scope.as_deref(),
        Some("configs/shared/inherited-root")
    );
    assert_eq!(
        cross.include,
        ["configs/shared/inherited-root/inherited/**/*.spec.ts"]
    );
    assert_eq!(
        cross.exclude,
        ["configs/shared/inherited-root/inherited-ignore/**"]
    );

    let local = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("cross-local"))
        .unwrap();
    assert_eq!(local.scope.as_deref(), Some("local-root"));
    assert_eq!(local.include, ["local-root/local/**/*.test.ts"]);
    assert_eq!(
        local.exclude,
        [
            "local-root/inherited-ignore/**",
            "local-root/local-ignore/**"
        ]
    );

    let merged = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("merged-extends"))
        .unwrap();
    assert_eq!(merged.scope.as_deref(), Some("merged-root"));
    assert_eq!(merged.include, ["merged-root/owned/**/*.test.ts"]);
    assert!(merged
        .vitest_setup
        .iter()
        .any(|setup| setup.specifier.as_deref() == Some("./merged-setup.ts")));

    let dynamic = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("merged-dynamic"))
        .unwrap();
    assert!(dynamic.vitest_setup.iter().any(|setup| {
        setup.unresolved_config_extends.as_deref() == Some("./vite.merged-dynamic.config.js")
    }));
    assert!(dynamic
        .vitest_setup
        .iter()
        .any(|setup| setup.config_extends_provenance));
}

fn assert_merged_provenance(
    project: &crate::integration_tests::types::ConfigProject,
    specifier: &str,
) {
    let setup = project
        .vitest_setup
        .iter()
        .find(|setup| setup.specifier.as_deref() == Some(specifier))
        .unwrap();
    assert!(setup
        .trigger_paths
        .iter()
        .any(|path| path.ends_with("setup-values.ts")));
    assert!(setup
        .trigger_paths
        .iter()
        .any(|path| path.ends_with("vitest.config.ts")));
}
