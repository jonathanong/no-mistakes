use super::*;
use std::collections::BTreeSet;

fn coverage_files(prefix: &str, suffix: &str) -> Vec<String> {
    let mut files: Vec<_> = std::fs::read_dir(fixture("coverage"))
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            entry
                .file_type()
                .unwrap()
                .is_file()
                .then(|| entry.file_name().to_string_lossy().into_owned())
        })
        .filter(|name| name.starts_with(prefix) && name.ends_with(suffix))
        .collect();
    files.sort();
    files
}

#[test]
fn playwright_config_parser_covers_project_defaults() {
    let root = fixture("coverage");
    let expected_errors = BTreeSet::from([
        "playwright.empty-match-invalid.ts",
        "playwright.empty-test-match.ts",
        "playwright.invalid.ts",
        "playwright.object-testignore-invalid.ts",
    ]);
    let mut policy_names = BTreeSet::new();

    for file in coverage_files("playwright.", ".ts") {
        let path = root.join(&file);
        let source = std::fs::read_to_string(&path).unwrap();
        let result = parse_playwright_fixture(&source, &path, &root);
        if expected_errors.contains(file.as_str()) {
            assert!(result.is_err(), "expected {file} to be rejected");
            continue;
        }
        let parsed = result.unwrap_or_else(|error| panic!("{file} should parse: {error:#}"));
        for project in parsed.into_projects(&root, &file) {
            if let Some(policy_name) = project.policy_name {
                policy_names.insert(policy_name);
            }
        }
    }

    for expected in [
        "absolute",
        "imported",
        "pw-root-call-import",
        "pw-object-call-destructure-body",
        "pw-member-spread-named",
    ] {
        assert!(
            policy_names.contains(expected),
            "missing Playwright policy {expected}"
        );
    }
    assert!(!policy_names.contains("root-spread-missing"));
}

#[test]
fn vitest_config_parser_covers_root_and_nested_projects() {
    let root = fixture("coverage");
    let expected_errors = BTreeSet::from([
        "vitest.empty-array-invalid.mts",
        "vitest.invalid.mts",
        "vitest.invalid-project.mts",
        "vitest.project-exclude-invalid.mts",
    ]);
    let mut policy_names = BTreeSet::new();

    for file in coverage_files("vitest.", ".mts") {
        let path = root.join(&file);
        let source = std::fs::read_to_string(&path).unwrap();
        let result = parse_vitest_fixture(&source, &path, &root);
        if expected_errors.contains(file.as_str()) {
            assert!(result.is_err(), "expected {file} to be rejected");
            continue;
        }
        let projects = result.unwrap_or_else(|error| panic!("{file} should parse: {error:#}"));
        for project in projects {
            if let Some(policy_name) = project.policy_name {
                policy_names.insert(policy_name);
            }
        }
    }

    for expected in [
        "root-vitest",
        "nested",
        "vitest-root-call-import",
        "vitest-object-call-destructure-body",
        "vitest-member-spread-named",
        "vitest-test-sourced-reexport",
    ] {
        assert!(
            policy_names.contains(expected),
            "missing Vitest policy {expected}"
        );
    }
    assert!(!policy_names.contains("vitest-root-spread-missing"));
}

#[test]
fn vitest_setup_dependencies_preserve_effective_project_ownership() {
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
    let setup = |name: &str| &project(name).vitest_setup;

    let inherited = setup("inherits");
    assert_eq!(inherited.len(), 4);
    assert_eq!(
        inherited
            .iter()
            .map(|dependency| dependency.field.as_str())
            .collect::<Vec<_>>(),
        vec!["setupFiles", "setupFiles", "setupFiles", "globalSetup"]
    );
    assert_eq!(inherited[0].specifier.as_deref(), Some("./setup/root.ts"));
    assert_eq!(
        inherited[0].resolved_path.as_deref(),
        Some(root.join("inherits/setup/root.ts").as_path())
    );
    assert_eq!(inherited[1].specifier, None);
    assert_eq!(
        inherited[2].specifier.as_deref(),
        Some("./setup/missing.ts")
    );
    assert_eq!(inherited[2].resolved_path, None);
    assert_eq!(
        inherited[3].resolved_path.as_deref(),
        Some(root.join("inherits/setup/global.mts").as_path())
    );
    assert!(inherited
        .iter()
        .all(|dependency| dependency.resolution_base == root.join("inherits")));
    assert!(inherited
        .iter()
        .all(|dependency| dependency.declaration_path == path));
    assert!(inherited
        .iter()
        .all(|dependency| dependency.declaration_line > 0));
    assert!(inherited.iter().any(|dependency| {
        dependency.specifier.is_none()
            && dependency
                .trigger_paths
                .contains(&root.join("config/setup-selector.ts"))
    }));

    let closure = setup("dynamic-closure");
    assert_eq!(closure.len(), 1, "{closure:#?}");
    assert!(closure[0].specifier.is_none());
    assert!(closure[0]
        .trigger_paths
        .contains(&root.join("config/dynamic-wrapper.ts")));
    assert!(closure[0]
        .trigger_paths
        .contains(&root.join("config/transitive-dynamic-helper.ts")));

    let overridden = setup("override");
    assert_eq!(overridden.len(), 1, "{overridden:#?}");
    assert_eq!(
        overridden[0].specifier.as_deref(),
        Some("./setup/override.js")
    );
    assert_eq!(
        overridden[0].resolved_path.as_deref(),
        Some(root.join("setup/override.js").as_path())
    );
    assert!(setup("cleared").is_empty());

    let imported = setup("imported");
    assert_eq!(imported.len(), 2);
    assert!(imported.iter().all(|dependency| {
        dependency.declaration_path == root.join("vitest.setup-imported.ts")
    }));
    assert_eq!(
        imported[0].specifier.as_deref(),
        Some("./setup/imported.cts")
    );
    assert_eq!(
        imported[0].resolved_path.as_deref(),
        Some(root.join("imported/setup/imported.cts").as_path())
    );
    assert_eq!(
        imported[1].resolved_path.as_deref(),
        Some(root.join("imported/setup/imported-global.cjs").as_path())
    );
    assert!(imported
        .iter()
        .all(|dependency| dependency.resolution_base == root.join("imported")));

    let string_project = setup("string-project");
    assert_eq!(string_project.len(), 2, "{string_project:#?}");
    assert!(string_project.iter().all(|dependency| {
        dependency.declaration_path == root.join("vitest.string-project.ts")
            && dependency.resolution_base == root.join("string-project")
    }));
    assert_eq!(
        string_project[0].resolved_path.as_deref(),
        Some(root.join("string-project/setup/string.ts").as_path())
    );
    assert_eq!(
        string_project[1].resolved_path.as_deref(),
        Some(root.join("string-project/setup/global.cjs").as_path())
    );

    let nested_string_project = setup("nested-string-default-root");
    assert_eq!(nested_string_project.len(), 2, "{nested_string_project:#?}");
    assert!(nested_string_project.iter().all(|dependency| {
        dependency.declaration_path == root.join("packages/foo/vitest.project.ts")
            && dependency.resolution_base == root.join("packages/foo")
    }));
    assert_eq!(
        nested_string_project[0].resolved_path.as_deref(),
        Some(root.join("packages/foo/setup.ts").as_path())
    );
    assert_eq!(
        nested_string_project[1].resolved_path.as_deref(),
        Some(root.join("packages/foo/global.ts").as_path())
    );
    assert_eq!(
        project("nested-string-default-root").scope.as_deref(),
        Some("packages/foo")
    );
    assert!(project("nested-string-default-root")
        .include
        .iter()
        .any(|include| include == "packages/foo/tests/**/*.test.ts"));
}
