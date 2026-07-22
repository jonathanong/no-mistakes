use super::*;
use std::collections::BTreeSet;

#[path = "parser_coverage/vitest_workspace.rs"]
mod workspace;

#[path = "parser_coverage/vitest_setup_branches.rs"]
mod vitest_setup_branches;

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

    for (name, file) in [
        ("local-member", "local-member.ts"),
        ("namespace-member", "namespace-member.ts"),
    ] {
        assert_eq!(setup(name).len(), 1, "{name}");
        assert_eq!(
            setup(name)[0].resolved_path.as_deref(),
            Some(root.join("setup").join(file).as_path()),
            "{name}"
        );
    }

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
    assert!(closure[0]
        .trigger_paths
        .contains(&root.join("config/runtime-star-helper.ts")));
    assert!(
        !closure[0]
            .trigger_paths
            .contains(&root.join("config/type-only-helper.ts")),
        "dynamic setup closures follow runtime re-exports but exclude type-only sources"
    );
    let mut reresolved = closure[0].clone();
    let tsconfig = test_support::tsconfig_without_config(&root);
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    crate::integration_tests::resolve_setup_dependencies(
        std::iter::once(&mut reresolved),
        &root.join("dynamic-closure"),
        &resolver,
    );
    assert!(reresolved
        .trigger_paths
        .contains(&root.join("config/dynamic-wrapper.ts")));
    assert!(reresolved
        .trigger_paths
        .contains(&root.join("config/runtime-star-helper.ts")));

    let cycle = setup("dynamic-cycle");
    assert_eq!(cycle.len(), 1, "{cycle:#?}");
    assert!(cycle[0].specifier.is_none());
    assert_eq!(cycle[0].trigger_paths, BTreeSet::from([path.clone()]));

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

    let imported_values = setup("imported-values");
    assert_eq!(imported_values.len(), 9, "{imported_values:#?}");
    assert!(imported_values
        .iter()
        .all(|dependency| dependency.specifier.is_some() && dependency.resolved_path.is_some()));
    assert!(imported_values.iter().take(2).all(|dependency| {
        dependency.declaration_path == root.join("config/imported-setup-values.ts")
    }));
    assert_eq!(
        imported_values[0].resolved_path.as_deref(),
        Some(
            root.join("imported-values/setup/imported-value.ts")
                .as_path()
        )
    );
    assert_eq!(
        imported_values[1].resolved_path.as_deref(),
        Some(
            root.join("imported-values/setup/imported-array.ts")
                .as_path()
        )
    );
    assert_eq!(
        imported_values[2].resolved_path.as_deref(),
        Some(
            root.join("imported-values/setup/imported-template.ts")
                .as_path()
        )
    );
    assert_eq!(
        imported_values
            .iter()
            .skip(3)
            .filter_map(|dependency| dependency.resolved_path.as_ref())
            .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
            .collect::<Vec<_>>(),
        [
            "default-imported.ts",
            "default-named.ts",
            "source-reexported.ts",
            "imported-reexported.ts",
            "barrel.ts",
            "template-default.ts",
        ],
    );

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

#[test]
fn vitest_setup_config_fallbacks_are_fixture_backed() {
    let source_fixture = &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-parser-coverage");
    let fixture = crate::test_support::materialize_saved_fixture(source_fixture);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let parsed = parse_vitest_fixture(&source, &path, &root).unwrap();
    assert_eq!(
        parsed
            .iter()
            .filter_map(|project| project.policy_name.as_deref())
            .collect::<Vec<_>>(),
        vec!["spread-setup", "named-dynamic", "directory-dynamic"],
    );
    assert!(parsed
        .iter()
        .all(|project| project.policy_name.as_deref() != Some("ignored-project")));
    let spread = parsed
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("spread-setup"))
        .unwrap();
    assert_eq!(spread.vitest_setup.len(), 1);
    assert_eq!(
        spread.vitest_setup[0].specifier.as_deref(),
        Some("./setup.ts")
    );
    let named = parsed
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("named-dynamic"))
        .unwrap();
    assert!(named.vitest_setup[0]
        .trigger_paths
        .contains(&root.join("dynamic.ts")));

    let tsconfig = test_support::tsconfig_without_config(&root);
    let visible = std::collections::HashSet::from([root.join("config")]);

    let projects =
        test_support::parse_vitest_from_visible(&source, &path, &root, &root, &tsconfig, &visible)
            .expect("directory and unresolved config candidates should be ignored");

    assert_eq!(projects.len(), 3);
    assert_eq!(
        projects
            .iter()
            .filter_map(|project| project.policy_name.as_deref())
            .collect::<Vec<_>>(),
        vec!["spread-setup", "named-dynamic", "directory-dynamic"],
    );
    let directory_dynamic = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("directory-dynamic"))
        .unwrap();
    assert!(directory_dynamic.vitest_setup[0]
        .trigger_paths
        .contains(&root.join("config")));
}

#[test]
fn vitest_project_string_entries_use_only_the_visible_config_universe() {
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

    let projects =
        test_support::parse_vitest_from_visible(&source, &path, &root, &root, &tsconfig, &visible)
            .unwrap();
    assert_eq!(
        projects
            .iter()
            .filter_map(|project| project.policy_name.as_deref())
            .collect::<Vec<_>>(),
        vec![
            "z",
            "folder",
            "imported-allowed",
            "inline-z",
            "inline-direct-spread",
            "a",
            "inline-a",
            "function-expression",
            "self",
            "direct",
            "glob",
            "named",
        ],
    );
    assert_eq!(
        projects
            .iter()
            .filter_map(|project| project.policy_name.as_deref())
            .take(6)
            .collect::<Vec<_>>(),
        vec![
            "z",
            "folder",
            "imported-allowed",
            "inline-z",
            "inline-direct-spread",
            "a",
        ],
        "string and inline projects retain their first-occurrence source order",
    );
    assert_eq!(
        projects
            .iter()
            .filter(|project| project.policy_name.as_deref() == Some("folder"))
            .count(),
        1,
        "the folder entry and wildcard overlap but produce one project",
    );
    assert!(projects
        .iter()
        .all(|project| project.policy_name.as_deref() != Some("excluded")));
    assert!(projects
        .iter()
        .all(|project| project.policy_name.as_deref() != Some("imported-excluded")));
    assert!(
        projects
            .iter()
            .all(|project| project.policy_name.as_deref() != Some("negated-outer")),
        "an imported negation must skip an outer config before its setup state is parsed"
    );
    assert!(
        projects.iter().all(|project| {
            !matches!(
                project.policy_name.as_deref(),
                Some(
                    "negated-local"
                        | "negated-imported"
                        | "negated-default-function"
                        | "negated-default-function-identifier"
                        | "negated-named-function"
                )
            )
        }),
        "local and imported static call returns must suppress outer setup configs"
    );
    let direct = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("direct"))
        .unwrap();
    assert_eq!(direct.vitest_setup.len(), 1);
    assert!(direct.vitest_setup[0]
        .resolved_path
        .as_ref()
        .is_some_and(|path| path.ends_with("projects/direct-setup.ts")));
}

#[test]
fn vitest_inline_setup_inheritance_requires_extends_true() {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/test-config/vitest-extends");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_vitest_fixture(&source, &path, &root).unwrap();

    for name in ["default", "false", "nonboolean", "spread-false-last"] {
        assert!(projects
            .iter()
            .find(|project| project.policy_name.as_deref() == Some(name))
            .unwrap()
            .vitest_setup
            .is_empty());
    }
    for name in ["true", "spread-true-last"] {
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
    let root_setup = merged
        .vitest_setup
        .iter()
        .find(|setup| setup.specifier.as_deref() == Some("./root-setup.ts"))
        .unwrap();
    assert!(root_setup
        .trigger_paths
        .iter()
        .any(|path| path.ends_with("setup-values.ts")));
    assert!(root_setup
        .trigger_paths
        .iter()
        .any(|path| path.ends_with("vitest.config.ts")));
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
