use super::*;

#[test]
fn commonjs_project_arrays_support_direct_requires_and_named_exports() {
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
            .unwrap_or_else(|| panic!("missing CommonJS project {name}"))
    };

    for (name, setup) in [
        ("cjs-direct-element", "direct-element.ts"),
        ("cjs-direct-spread", "direct-spread.ts"),
        ("cjs-named-member", "named-member.ts"),
        ("cjs-named-object", "named-object.ts"),
        ("cjs-named-replacement", "named-replacement.ts"),
    ] {
        let project = project(name);
        assert_eq!(project.vitest_setup.len(), 1, "{name}: {project:#?}");
        assert_eq!(
            project.vitest_setup[0]
                .resolved_path
                .as_deref()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str()),
            Some(setup),
            "{name}: {project:#?}"
        );
    }
    for unsupported in [
        "cjs-named-stale",
        "cjs-named-detached",
        "cjs-computed-project",
        "cjs-module-default-member",
        "cjs-exports-default-member",
        "cjs-require-excluded",
        "cjs-named-excluded",
        "cjs-alias-excluded",
        "cjs-chain-excluded",
        "cjs-cycle-excluded",
        "cjs-named-alias-excluded",
        "cjs-named-object-excluded",
        "cjs-named-reexport-excluded",
    ] {
        assert!(
            !projects
                .iter()
                .any(|project| project.policy_name.as_deref() == Some(unsupported)),
            "unsupported CommonJS form leaked into projects: {unsupported}"
        );
    }
}
