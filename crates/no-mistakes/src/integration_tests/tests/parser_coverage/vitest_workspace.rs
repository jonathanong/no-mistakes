use super::*;
use crate::integration_tests::types::Framework;
use std::collections::BTreeSet;

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
        for project in result.unwrap_or_else(|error| panic!("{file} should parse: {error:#}")) {
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

fn saved_fixture(name: &str) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config")
        .join(name);
    crate::test_support::materialize_saved_fixture(&source)
}

#[test]
fn explicitly_configured_json_workspaces_parse_objects_and_string_projects() {
    let fixture = saved_fixture("vitest-workspace-json");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let visible = crate::codebase::ts_source::discover_visible_paths(&root);
    let config = crate::config::v2::load_v2_config_from_visible(&root, None, &visible).unwrap();
    let tsconfig = test_support::tsconfig_without_config(&root);
    crate::ast::begin_parse_count(&root);
    let projects = crate::integration_tests::project_config::load_projects_from_visible(
        &root,
        Framework::Vitest,
        config.tests.vitest.configs.as_ref(),
        &visible,
        &tsconfig,
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(
        projects
            .iter()
            .filter_map(|project| project.policy_name.as_deref())
            .collect::<Vec<_>>(),
        ["json-inline", "json-string"]
    );
    assert!(projects.iter().all(|project| project.workspace));
    assert!(projects
        .iter()
        .filter(|project| project.policy_name.is_some())
        .all(|project| {
            project.vitest_setup.len() == 1 && project.vitest_setup[0].resolved_path.is_some()
        }));
    let inline = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("json-inline"))
        .unwrap();
    assert_eq!(inline.vitest_setup[0].field.as_str(), "globalSetup");
    assert_eq!(
        inline.vitest_setup[0].specifier.as_deref(),
        Some("./setup.ts")
    );
    assert_eq!(counts.get(&root.join("vitest.workspace.json")), None);
    assert_eq!(counts.get(&root.join("vitest.projects.json")), None);
    assert_eq!(
        counts.get(&root.join("string-project/vitest.config.ts")),
        Some(&1)
    );
    assert!(
        crate::integration_tests::project_config::load_projects_from_visible(
            &root,
            Framework::Vitest,
            None,
            &visible,
            &tsconfig,
        )
        .unwrap()
        .is_empty(),
        "JSON project arrays remain explicit-only"
    );
}

#[test]
fn vitest_setup_does_not_resolve_a_declaration_file_as_runtime() {
    let fixture = saved_fixture("vitest-declaration-only");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let setup = &parse_vitest_fixture(&source, &path, &root).unwrap()[0].vitest_setup[0];

    assert_eq!(setup.specifier.as_deref(), Some("./declaration-only"));
    assert!(setup.resolved_path.is_none());
    assert!(!setup.trigger_paths.iter().any(|path| {
        path.file_name()
            .is_some_and(|name| name == "declaration-only.d.ts")
    }));
}

#[test]
fn vitest_absolute_setup_paths_resolve_runtime_closures_but_not_declarations() {
    let fixture = saved_fixture("vitest-absolute-setup");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path)
        .unwrap()
        .replace(
            "__ABSOLUTE_RUNTIME_SETUP__",
            &root.join("absolute-setup.ts").to_string_lossy(),
        )
        .replace(
            "__ABSOLUTE_DECLARATION_SETUP__",
            &root.join("absolute-declaration.d.ts").to_string_lossy(),
        );
    let project = &parse_vitest_fixture(&source, &path, &root).unwrap()[0];
    let runtime = project
        .vitest_setup
        .iter()
        .find(|setup| {
            setup
                .specifier
                .as_deref()
                .is_some_and(|specifier| Path::new(specifier).is_absolute())
        })
        .unwrap();
    assert_eq!(
        runtime.resolved_path.as_deref(),
        Some(root.join("absolute-setup.ts").as_path())
    );
    assert!(runtime
        .transitive_trigger_paths
        .contains(&root.join("absolute-helper.ts")));
    assert!(project
        .vitest_setup
        .iter()
        .any(|setup| setup.specifier.as_deref().is_some_and(|specifier| {
            specifier.ends_with("absolute-declaration.d.ts") && setup.resolved_path.is_none()
        })));
}

fn assert_workspace_project(name: &str, extension: &str, project: &str) {
    let fixture = saved_fixture(name);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join(format!("vitest.workspace.{extension}"));
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_vitest_fixture(&source, &path, &root).unwrap();
    assert_eq!(projects[0].policy_name.as_deref(), Some(project));
    assert_eq!(
        projects[0].vitest_setup[0].resolved_path.as_deref(),
        Some(root.join("workspace-setup.ts").as_path())
    );
}

#[test]
fn vitest_workspace_default_array_keeps_project_setup_ownership() {
    assert_workspace_project("vitest-workspace-default", "ts", "workspace-project");
}

#[test]
fn vitest_workspace_direct_default_array_is_parsed() {
    assert_workspace_project(
        "vitest-workspace-direct-array",
        "ts",
        "direct-array-project",
    );
}

#[test]
fn vitest_workspace_named_export_reexported_as_default_is_parsed() {
    assert_workspace_project(
        "vitest-workspace-named-reexport",
        "ts",
        "named-reexport-project",
    );
}

#[test]
fn vitest_workspace_follows_star_and_named_export_reexports() {
    let fixture = saved_fixture("vitest-workspace-export-forms");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    for (file, project) in [
        ("vitest.workspace.ts", "star-export-project"),
        ("vitest.projects.ts", "named-export-project"),
    ] {
        let path = root.join(file);
        let source = std::fs::read_to_string(&path).unwrap();
        let projects = parse_vitest_fixture(&source, &path, &root).unwrap();
        assert_eq!(projects[0].policy_name.as_deref(), Some(project), "{file}");
    }
}

#[test]
fn vitest_workspace_commonjs_exports_keep_workspace_setup_ownership() {
    assert_workspace_project(
        "vitest-workspace-commonjs-array",
        "cjs",
        "commonjs-array-project",
    );
    assert_workspace_project(
        "vitest-workspace-commonjs-define",
        "cts",
        "commonjs-define-project",
    );
}

#[test]
fn vitest_projects_files_are_project_array_sources_for_all_runtime_extensions() {
    let fixture = saved_fixture("vitest-projects-extensions");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    for extension in ["ts", "mts", "cts", "js", "mjs", "cjs"] {
        let path = root.join(format!("vitest.projects.{extension}"));
        let source = std::fs::read_to_string(&path).unwrap();
        let projects = parse_vitest_fixture(&source, &path, &root).unwrap();
        assert_eq!(
            projects[0].policy_name.as_deref(),
            Some(format!("projects-{extension}").as_str()),
            "{extension}",
        );
        assert_eq!(
            projects[0].vitest_setup[0].resolved_path.as_deref(),
            Some(root.join("setup.ts").as_path()),
            "{extension}",
        );
    }
}

#[test]
fn arbitrary_default_call_is_not_a_workspace_config() {
    let fixture = saved_fixture("vitest-workspace-default");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("not-workspace.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();

    assert!(parse_vitest_fixture(&source, &path, &root)
        .unwrap()
        .is_empty());
}

#[test]
fn vitest_project_glob_accepts_config_suffixes() {
    let fixture = saved_fixture("vitest-config-suffix-glob");
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
        ["e2e-suffix", "unit-suffix"]
    );
}

#[test]
fn vitest_folder_globs_only_parse_configs_in_matched_roots() {
    let fixture = saved_fixture("vitest-project-folder-glob");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let visible = crate::codebase::ts_source::discover_visible_paths(&root)
        .into_iter()
        .collect();
    let tsconfig = test_support::tsconfig_without_config(&root);
    let parse = |config: &str| {
        let path = root.join(config);
        let source = std::fs::read_to_string(&path).unwrap();
        test_support::parse_vitest_from_visible(&source, &path, &root, &root, &tsconfig, &visible)
            .unwrap()
    };

    let broad = parse("vitest.config.ts");
    assert_eq!(
        broad
            .iter()
            .filter_map(|project| project.policy_name.as_deref())
            .collect::<Vec<_>>(),
        ["direct-project"]
    );
    assert_eq!(
        broad
            .iter()
            .filter(|project| project.policy_name.is_none())
            .filter_map(|project| project.scope.as_deref())
            .collect::<Vec<_>>(),
        [
            "packages/arbitrary",
            "packages/business",
            "packages/configless",
            "packages/custom",
        ],
        "folder globs retain configless visible roots as default projects"
    );
    assert!(
        broad
            .iter()
            .all(|project| project.scope.as_deref() != Some("packages/skip")),
        "the root negation must filter configless roots from imported folder globs"
    );
    let configless = broad
        .iter()
        .find(|project| project.scope.as_deref() == Some("packages/configless"))
        .expect("synthesized configless project");
    assert!(
        configless
            .include
            .iter()
            .any(|include| include == "packages/configless/**/*.test.ts"),
        "configless folders must not inherit the aggregate root include"
    );
    let nested = parse("vitest.business.config.ts");
    assert_eq!(
        nested
            .iter()
            .filter_map(|project| project.policy_name.as_deref())
            .collect::<Vec<_>>(),
        ["nested-business-project"]
    );
    let arbitrary = parse("vitest.arbitrary.config.ts");
    assert_eq!(
        arbitrary
            .iter()
            .filter_map(|project| project.policy_name.as_deref())
            .collect::<Vec<_>>(),
        ["arbitrary-project-file"],
        "an explicit project-file glob may use any supported runtime filename"
    );
    let custom = parse("vitest.custom-project.config.ts");
    assert_eq!(
        custom
            .iter()
            .filter_map(|project| project.policy_name.as_deref())
            .collect::<Vec<_>>(),
        ["custom-extension-project"],
        "an explicit project file may use an obscure extension with TS parsing fallback"
    );
    let exact_folder = parse("vitest.exact-folder.config.ts");
    assert!(
        exact_folder
            .iter()
            .all(|project| project.policy_name.as_deref() != Some("nested-business-project")),
        "an exact folder entry must not parse a nested Vitest config"
    );
    assert_eq!(
        exact_folder
            .iter()
            .filter_map(|project| project.scope.as_deref())
            .collect::<Vec<_>>(),
        ["packages/business"]
    );
}

#[test]
fn commonjs_require_bindings_ignore_incomplete_static_forms() {
    let fixture = saved_fixture("vitest-commonjs-negative");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.cjs");
    let source = std::fs::read_to_string(&path).unwrap();

    let projects = parse_vitest_fixture(&source, &path, &root).unwrap();
    assert_eq!(
        projects[0].policy_name.as_deref(),
        Some("commonjs-negative-bindings")
    );
    assert!(projects[0].vitest_setup.is_empty());
}

#[test]
fn explicit_custom_project_syntax_errors_are_reported() {
    let fixture = saved_fixture("vitest-custom-project-error");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let visible = crate::codebase::ts_source::discover_visible_paths(&root)
        .into_iter()
        .collect();
    let tsconfig = test_support::tsconfig_without_config(&root);

    assert!(test_support::parse_vitest_from_visible(
        &source, &path, &root, &root, &tsconfig, &visible,
    )
    .is_err());
}

#[test]
fn vitest_parent_relative_project_strings_and_globs_stay_in_visible_universe() {
    let fixture = saved_fixture("vitest-parent-projects");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("configs/vitest.config.ts");
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
        ["parent-e2e", "parent-unit"]
    );
}
