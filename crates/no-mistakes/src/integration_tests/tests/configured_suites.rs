use super::*;

#[test]
fn coverage_fixture_exercises_parser_and_resolution_variants() {
    let root = fixture("coverage");
    let findings = check(&root, None).unwrap();
    assert!(findings.iter().any(|finding| {
        finding.framework == "vitest"
            && finding.suite == "root-vitest.openai"
            && finding.test_name.as_deref() == Some("uses declared function")
            && finding.integration.as_deref() == Some("openai")
    }));
    assert!(findings.iter().any(|finding| {
        finding.suite == "root-vitest.openai"
            && finding.test_name.as_deref() == Some("uses namespace function")
            && finding.integration.as_deref() == Some("openai")
    }));
    assert!(findings
        .iter()
        .all(|finding| finding.suite != "nested-suite"));
}

#[test]
fn invalid_suite_project_and_missing_config_are_rejected() {
    let missing = check(&fixture("missing-config"), None).unwrap_err();
    assert!(missing.to_string().contains("config does not exist"));

    let unknown = check(&fixture("unknown-project"), None).unwrap_err();
    assert!(unknown
        .to_string()
        .contains("vitest integration policy references unknown project missing"));
}

#[test]
fn configured_suites_cover_matching_variants() {
    let root = fixture("coverage");
    let config = config_snippet("configured-suites.yml");
    let suites = test_support::configured_suites(&root, &config).unwrap();
    assert!(suites.iter().any(|suite| suite.name == "inherits.openai"));
    assert!(suites.iter().any(|suite| suite.name == "absolute.openai"
        && suite.include == vec!["/tmp/no-mistakes-absolute-tests/**/*.spec.ts"]));
    assert!(suites
        .iter()
        .any(|suite| suite.name == "root-vitest.openai"));

    let config = config_snippet("missing-playwright-config.yml");
    let err = test_support::configured_suites(&root, &config).unwrap_err();
    assert!(err.to_string().contains("config does not exist"));

    let config = config_snippet("empty-policy-with-missing-config.yml");
    assert!(test_support::configured_suites(&root, &config)
        .unwrap()
        .is_empty());

    let config = config_snippet("mixed-empty-and-nonempty-policy.yml");
    let suites = test_support::configured_suites(&root, &config).unwrap();
    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].name, "root-vitest.openai");

    let config = config_snippet("explicit-project-policy.yml");
    let suites = test_support::configured_suites(&root, &config).unwrap();
    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].name, "explicit.openai");
    assert_eq!(suites[0].include, vec!["explicit/**/*.test.ts"]);
    assert_eq!(suites[0].exclude, vec!["explicit/**/*.mock.test.ts"]);

    assert!(
        project_config::load_projects(&root, types::Framework::Vitest, None)
            .unwrap()
            .iter()
            .any(|project| project.config.as_deref() == Some("vitest.projects.mts"))
    );
    let commonjs_root = fixture("cjs-cts-configs");
    assert!(
        project_config::load_projects(&commonjs_root, types::Framework::Vitest, None)
            .unwrap()
            .iter()
            .any(
                |project| project.config.as_deref() == Some("vitest.config.cts")
                    && project.policy_name.as_deref() == Some("unit")
            )
    );
    assert!(
        project_config::load_projects(&commonjs_root, types::Framework::Playwright, None)
            .unwrap()
            .iter()
            .any(|project| project.config.as_deref() == Some("playwright.config.cjs"))
    );
    assert!(
        !project_config::load_projects(&fixture("basic"), types::Framework::Playwright, None)
            .unwrap()
            .is_empty()
    );
    let visible = crate::codebase::ts_source::VisiblePathSnapshot::new(&root).paths_for(&root);
    assert!(
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &visible)
            .unwrap()
            .base_url
            .is_some()
    );
    assert!(project_config::build_globset(&["[".to_string()]).is_err());
    assert!(!project_config::load_projects(
        &root,
        types::Framework::Playwright,
        Some(&crate::config::v2::schema::StringOrList::One(
            "playwright.projects.ts".to_string()
        )),
    )
    .unwrap()
    .is_empty());
    assert!(project_config::load_projects(
        &root,
        types::Framework::Playwright,
        Some(&crate::config::v2::schema::StringOrList::One(
            "playwright.invalid.ts".to_string()
        )),
    )
    .is_err());
    let package_root = fixture("vitest-package-tsconfig");
    let package_snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&package_root);
    let package_config_dir = package_root.join("packages/app");
    let package_tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        None,
        &package_config_dir,
        &package_snapshot.paths_for(&package_config_dir),
    )
    .unwrap();
    let package_projects = project_config::load_projects_from_visible(
        &package_root,
        types::Framework::Vitest,
        Some(&crate::config::v2::schema::StringOrList::One(
            "packages/app/vitest.config.mts".to_string(),
        )),
        &package_snapshot.paths_for(&package_root),
        &package_tsconfig,
    )
    .unwrap();
    assert!(package_projects.iter().any(|project| {
        project.config.as_deref() == Some("packages/app/vitest.config.mts")
            && project.policy_name.as_deref() == Some("package")
            && project.include == vec!["packages/app/package/**/*.test.ts"]
    }));
    let invalid_tsconfig_root = fixture("invalid-vitest-tsconfig");
    let err = project_config::load_projects(
        &invalid_tsconfig_root,
        types::Framework::Vitest,
        Some(&crate::config::v2::schema::StringOrList::One(
            "vitest.config.mts".to_string(),
        )),
    )
    .unwrap_err();
    assert!(format!("{err:#}").contains("loading tsconfig"));
    let invalid_playwright_tsconfig_root = fixture("playwright-invalid-tsconfig");
    let playwright_err = project_config::load_projects(
        &invalid_playwright_tsconfig_root,
        types::Framework::Playwright,
        Some(&crate::config::v2::schema::StringOrList::One(
            "playwright.config.ts".to_string(),
        )),
    )
    .unwrap_err();
    assert!(format!("{playwright_err:#}").contains("loading tsconfig"));

    let config = config_snippet("missing-config-and-project.yml");
    let err = test_support::configured_suites(&root, &config).unwrap_err();
    assert!(err.to_string().contains("config does not exist"));
}

#[test]
fn configured_suites_reject_duplicate_project_names() {
    let root = fixture("duplicate-projects");
    let config = config_snippet("duplicate-vitest-project-policy.yml");

    let err = test_support::configured_suites(&root, &config).unwrap_err();

    assert!(err
        .to_string()
        .contains("vitest integration policy references ambiguous project unit"));
}

#[test]
fn configured_suites_support_vitest_commonjs_auto_discovery() {
    let root = fixture("vitest-cjs-config");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();

    let suites = test_support::configured_suites(&root, &config).unwrap();

    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].name, "unit.openai");
    assert_eq!(suites[0].include, vec!["unit/**/*.test.ts"]);
}

#[test]
fn analyze_files_covers_import_and_function_shapes() {
    let file = fixture_file("coverage", "src/source.test.ts");
    let missing = fixture_file("coverage", "src/does-not-exist.ts");
    let analyses = test_support::analyze_files(&[missing, file.clone()]).unwrap();
    let analysis = analyses.get(&file).unwrap();

    assert!(analysis.imports.contains_key("defaultCall"));
    assert!(analysis.imports.contains_key("renamedCall"));
    assert!(analysis.imports.contains_key("helperNamespace"));
    assert!(analysis.functions.contains_key("declaredIntegration"));
    assert!(analysis.functions.contains_key("arrowIntegration"));
    assert!(analysis.functions.contains_key("functionIntegration"));
    assert!(analysis.functions.contains_key("exportedDeclared"));
    assert!(analysis.functions.contains_key("exportedArrow"));
    assert!(analysis.functions.contains_key("exportedFunction"));
    assert!(analysis
        .tests
        .iter()
        .any(|test| test.name.as_deref() == Some("uses declared function")));
}

#[test]
fn call_helpers_cover_non_test_and_member_variants() {
    let path = fixture_file("coverage", "src/calls.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let mut collector = CallAssertions::default();
        collector.visit_program(program);
        assert!(collector.saw_describe_as_non_test);
        assert!(collector.saw_non_string_test);
        assert!(collector.saw_function_callback);
        assert!(collector.saw_imported_member_call);
        assert!(collector.saw_non_callback_argument);
    })
    .unwrap();
}

#[derive(Default)]
struct CallAssertions {
    saw_describe_as_non_test: bool,
    saw_non_string_test: bool,
    saw_function_callback: bool,
    saw_imported_member_call: bool,
    saw_non_callback_argument: bool,
}

impl<'a> Visit<'a> for CallAssertions {
    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        let path = crate::ast::expression_path(&call.callee);
        if path
            .as_ref()
            .is_some_and(|path| path == &["test", "describe"])
        {
            self.saw_describe_as_non_test = calls::test_name(call).is_none();
        }
        if path.as_ref().is_some_and(|path| path == &["test"]) && calls::test_name(call).is_none() {
            self.saw_non_string_test = true;
            self.saw_non_callback_argument = calls::callback_argument(call).is_none();
            assert!(calls::collect_calls(call.arguments.first().unwrap()).is_empty());
        }
        if calls::test_name(call).as_deref() == Some("function callback") {
            let (argument, _) = calls::callback_argument(call).unwrap();
            let collected = calls::collect_calls(argument);
            self.saw_function_callback = true;
            self.saw_imported_member_call = collected.iter().any(
                |target| matches!(target, types::CallTarget::Imported { local } if local == "foo"),
            );
        }
        walk::walk_call_expression(self, call);
    }
}

#[test]
fn swift_framework_string_is_stable() {
    assert_eq!(types::Framework::Dotnet.as_str(), "dotnet");
    assert_eq!(types::Framework::Swift.as_str(), "swift");
    assert_eq!(types::Framework::Cargo.as_str(), "cargo");
}
