use super::*;
use oxc_ast_visit::{walk, Visit};
use oxc_span::Span;
use std::path::{Path, PathBuf};

mod config_parsers;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests")
            .join(name)
            .join("fixture"),
    )
}

fn fixture_file(name: &str, file: &str) -> PathBuf {
    fixture(name).join(file)
}

fn config_snippet(name: &str) -> crate::config::v2::schema::NoMistakesConfig {
    let yaml = std::fs::read_to_string(fixture_file("config-snippets", name)).unwrap();
    serde_yaml::from_str(&yaml).unwrap()
}

fn parse_vitest_fixture(
    source: &str,
    path: &Path,
    root: &Path,
) -> anyhow::Result<Vec<types::ConfigProject>> {
    let tsconfig = tsconfig_without_config(root);
    test_config::vitest::parse_from_path(source, path, root, root, &tsconfig)
}

fn parse_playwright_fixture(
    source: &str,
    path: &Path,
    config_dir: &Path,
) -> anyhow::Result<test_config::playwright::ParsedPlaywrightConfig> {
    let tsconfig = tsconfig_without_config(config_dir);
    test_config::playwright::parse_from_path(source, path, config_dir, &tsconfig)
}

#[test]
fn check_reports_integration_policy_violations() {
    let findings = check(&fixture("basic"), None).unwrap();
    let messages: Vec<_> = findings
        .iter()
        .map(|finding| {
            (
                finding.framework.as_str(),
                finding.suite.as_str(),
                finding.file.as_str(),
                finding.test_name.as_deref(),
                finding.integration.as_deref(),
            )
        })
        .collect();

    assert!(messages.contains(&(
        "vitest",
        "unit.unit",
        "backend/unit.test.mts",
        Some("direct integration in unit suite"),
        Some("openai"),
    )));
    assert!(messages.contains(&(
        "vitest",
        "unit.unit",
        "backend/unit.test.mts",
        Some("helper integration in unit suite"),
        Some("openai"),
    )));
    assert!(messages.contains(&(
        "vitest",
        "unit.unit",
        "backend/unit.test.mts",
        Some("expression helper integration in unit suite"),
        Some("openai"),
    )));
    assert!(messages.contains(&(
        "vitest",
        "mixed.openai",
        "mixed/mixed.test.mts",
        Some("wrong integration still fails in non-strict suite"),
        Some("anthropic"),
    )));
    assert!(messages.contains(&(
        "vitest",
        "mixed.openai",
        "mixed/mixed.test.mts",
        Some("wrong integration fails even when allowed integration is also called"),
        Some("anthropic"),
    )));
    assert!(messages.contains(&(
        "playwright",
        "pw-unit.unit",
        "playwright/unit/unit.spec.ts",
        Some("playwright helper integration in unit suite"),
        Some("openai"),
    )));
    assert_eq!(findings.len(), 6);
}

#[test]
fn multiple_integration_suites_for_one_project_share_project_scope_once() {
    let root = fixture("basic");
    let config = fixture_file("basic", "multiple-integration-suites.no-mistakes.yml");
    let findings = check(&root, Some(&config)).unwrap();

    assert_eq!(findings, Vec::new());
}

#[test]
fn empty_project_policy_is_allowed() {
    let config = config_snippet("empty-project-policy.yml");
    config::validate_config(&config).unwrap();
}

#[test]
fn invalid_empty_integration_suites_is_rejected() {
    let config = config_snippet("invalid-empty-integration-suites.yml");
    let err = config::validate_config(&config).unwrap_err();
    assert!(err
        .to_string()
        .contains("tests.vitest.projects.web.integration_suites.openai"));
}

#[test]
fn test_project_exclude_requires_include() {
    let config: crate::config::v2::schema::NoMistakesConfig = serde_yaml::from_str(
        r#"
tests:
  vitest:
    projects:
      web:
        exclude: ["web/generated/**"]
"#,
    )
    .unwrap();
    let err = config::validate_config(&config).unwrap_err();
    assert!(err
        .to_string()
        .contains("tests.vitest.projects.web.exclude requires include"));
}

#[test]
fn annotation_requires_one_valid_value() {
    let valid = "const f = /* no-mistakes: integration=openai */ async () => {}";
    let valid_start = valid.find("async").unwrap() as u32;
    assert_eq!(
        calls::integration_annotation_before(valid, Span::new(valid_start, valid_start + 5))
            .as_deref(),
        Some("openai")
    );

    let jsdoc = "/**\n * no-mistakes: integration: aws\n */\nasync function f() {}";
    let jsdoc_start = jsdoc.find("async").unwrap() as u32;
    assert_eq!(
        calls::integration_annotation_before(jsdoc, Span::new(jsdoc_start, jsdoc_start + 5))
            .as_deref(),
        Some("aws")
    );

    let invalid = "const f = /* no-mistakes: integration=openai,anthropic */ async () => {}";
    let invalid_start = invalid.find("async").unwrap() as u32;
    assert!(calls::integration_annotation_before(
        invalid,
        Span::new(invalid_start, invalid_start + 5)
    )
    .is_none());
}

#[test]
fn conditional_vitest_wrappers_are_detected_as_tests() {
    let source = "it.skipIf(!process.env.OPENAI_API_KEY)('real openai', async () => {})";
    crate::ast::with_program(Path::new("conditional.test.mts"), source, |program, _| {
        let mut names = Vec::new();
        struct Collector<'a>(&'a mut Vec<String>);
        impl<'a> Visit<'a> for Collector<'_> {
            fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
                if let Some(name) = calls::test_name(call) {
                    self.0.push(name);
                }
                walk::walk_call_expression(self, call);
            }
        }
        Collector(&mut names).visit_program(program);
        assert_eq!(names, vec!["real openai"]);
    })
    .unwrap();
}

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
    let suites = config::configured_suites(&root, &config).unwrap();
    assert!(suites.iter().any(|suite| suite.name == "inherits.openai"));
    assert!(suites.iter().any(|suite| suite.name == "absolute.openai"
        && suite.include == vec!["/tmp/no-mistakes-absolute-tests/**/*.spec.ts"]));
    assert!(suites
        .iter()
        .any(|suite| suite.name == "root-vitest.openai"));

    let config = config_snippet("missing-playwright-config.yml");
    let err = config::configured_suites(&root, &config).unwrap_err();
    assert!(err.to_string().contains("config does not exist"));

    let config = config_snippet("empty-policy-with-missing-config.yml");
    assert!(config::configured_suites(&root, &config)
        .unwrap()
        .is_empty());

    let config = config_snippet("mixed-empty-and-nonempty-policy.yml");
    let suites = config::configured_suites(&root, &config).unwrap();
    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].name, "root-vitest.openai");

    let config = config_snippet("explicit-project-policy.yml");
    let suites = config::configured_suites(&root, &config).unwrap();
    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].name, "explicit.openai");
    assert_eq!(suites[0].include, vec!["explicit/**/*.test.ts"]);
    assert_eq!(suites[0].exclude, vec!["explicit/**/*.mock.test.ts"]);

    assert!(
        project_config::load_projects(&root, types::Framework::Vitest, None)
            .unwrap()
            .is_empty()
    );
    let commonjs_root = fixture("cjs-cts-configs");
    assert!(
        project_config::load_projects(&commonjs_root, types::Framework::Vitest, None)
            .unwrap()
            .iter()
            .any(
                |project| project.config.as_deref() == Some("vitest.config.cts")
                    && project.name.as_deref() == Some("unit")
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
    assert!(project_config::resolve_tsconfig(&root)
        .unwrap()
        .base_url
        .is_some());
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
    let package_projects = project_config::load_projects(
        &package_root,
        types::Framework::Vitest,
        Some(&crate::config::v2::schema::StringOrList::One(
            "packages/app/vitest.config.mts".to_string(),
        )),
    )
    .unwrap();
    assert!(package_projects.iter().any(|project| {
        project.config.as_deref() == Some("packages/app/vitest.config.mts")
            && project.name.as_deref() == Some("package")
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
    let err = config::configured_suites(&root, &config).unwrap_err();
    assert!(err.to_string().contains("config does not exist"));
}

#[test]
fn configured_suites_reject_duplicate_project_names() {
    let root = fixture("duplicate-projects");
    let config = config_snippet("duplicate-vitest-project-policy.yml");

    let err = config::configured_suites(&root, &config).unwrap_err();

    assert!(err
        .to_string()
        .contains("vitest integration policy references ambiguous project unit"));
}

#[test]
fn configured_suites_support_vitest_commonjs_auto_discovery() {
    let root = fixture("vitest-cjs-config");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();

    let suites = config::configured_suites(&root, &config).unwrap();

    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].name, "unit.openai");
    assert_eq!(suites[0].include, vec!["unit/**/*.test.ts"]);
}

#[test]
fn analyze_files_covers_import_and_function_shapes() {
    let file = fixture_file("coverage", "src/source.test.ts");
    let missing = fixture_file("coverage", "src/does-not-exist.ts");
    let analyses = analysis::analyze_files(&[missing, file.clone()]).unwrap();
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
fn playwright_config_parser_covers_project_defaults() {
    let root = fixture("coverage");
    let path = root.join("playwright.projects.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let parsed = parse_playwright_fixture(&source, &path, &root).unwrap();
    let projects = parsed.into_projects(&root, "playwright.projects.ts");

    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("absolute")
            && project.include == vec!["/tmp/no-mistakes-absolute-tests/**/*.spec.ts"]
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("inherits")
            && project
                .exclude
                .iter()
                .any(|glob| glob.ends_with("root-ignore.ts"))
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("imported")
            && project.include == vec!["imported/**/*.imported.spec.ts"]
            && project
                .exclude
                .iter()
                .any(|glob| glob.ends_with("imported/**/*.skip.ts"))
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("imported-spread")
            && project.include == vec!["imported-spread/**/*.imported-spread.spec.ts"]
            && project
                .exclude
                .iter()
                .any(|glob| glob.ends_with("imported-spread/**/*.imported-spread.skip.ts"))
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("nested-imported-local-spread")
            && project.include == vec!["imported-spread/**/*.imported-spread.spec.ts"]
            && project
                .exclude
                .iter()
                .any(|glob| glob.ends_with("imported-spread/**/*.imported-spread.skip.ts"))
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("default-imported-spread")
            && project.include
                == vec!["default-imported-spread/**/*.default-imported-spread.spec.ts"]
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("reexported-spread")
            && project.include == vec!["reexported-spread/**/*.reexported-spread.spec.ts"]
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("star-reexported-spread")
            && project.include == vec!["reexported-spread/**/*.reexported-spread.spec.ts"]
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("type-star-spread")
            && project.include == vec!["type-star-spread/**/*.spec.ts"]
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("constant-spread")
            && project.include == vec!["constant-spread/**/*.constant-spread.spec.ts"]
    }));
    assert!(!projects.iter().any(|project| {
        project.name.as_deref() == Some("ambiguous-object-spread")
            && project
                .include
                .iter()
                .any(|glob| glob.contains("ambiguous-object-spread"))
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("nested-reexported-spread")
            && project.include
                == vec!["nested-reexported-spread/**/*.nested-reexported-spread.spec.ts"]
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("namespace-spread")
            && project.include == vec!["namespace-spread/**/*.namespace-spread.spec.ts"]
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("local-alias-spread")
            && project.include == vec!["local-alias-spread/**/*.local-alias-spread.spec.ts"]
    }));
    assert!(!projects.iter().any(|project| {
        project.name.as_deref() == Some("call-spread-ignored")
            && project.include == vec!["call-spread-ignored/**/*.spec.ts"]
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("trailing-imported-spread")
            && project.include
                == vec!["trailing-imported-spread/**/*.trailing-imported-spread.spec.ts"]
            && project.exclude.iter().any(|glob| {
                glob.ends_with("trailing-imported-spread/**/*.trailing-imported-spread.skip.ts")
            })
    }));
    assert!(projects
        .iter()
        .any(|project| { project.name.as_deref() == Some("defensive-spreads") }));
    assert!(!projects
        .iter()
        .any(|project| project.name.as_deref() == Some("nested-array-should-not-flatten")));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("factory")
            && project.include == vec!["factory/**/*.factory.spec.ts"]
    }));
    assert!(projects.iter().any(|project| {
        project.name.as_deref() == Some("wrapped")
            && project.include == vec!["root/wrapped/**/*.spec.ts"]
    }));

    let empty_path = root.join("playwright.empty.ts");
    let empty = std::fs::read_to_string(&empty_path).unwrap();
    let parsed = parse_playwright_fixture(&empty, &empty_path, &root).unwrap();
    assert_eq!(parsed.into_projects(&root, "playwright.empty.ts").len(), 1);

    let parsed = parse_playwright_fixture(&empty, &empty_path, "relative".as_ref()).unwrap();
    assert!(parsed.into_projects(&root, "relative.ts")[0].include[0].starts_with("relative/"));

    let edge_path = root.join("playwright.edge.ts");
    let edge_source = std::fs::read_to_string(&edge_path).unwrap();
    let edge = parse_playwright_fixture(&edge_source, &edge_path, &root)
        .unwrap()
        .into_projects(&root, "playwright.edge.ts");
    for name in [
        "pw-parenthesized",
        "pw-wrapped-helper",
        "pw-local-member-array",
        "pw-function-expression",
        "pw-block-arrow",
        "pw-top-level-function",
        "pw-named-var",
        "pw-named-function",
        "pw-local-alias",
        "pw-local-function",
        "pw-destructured",
        "pw-aliased-destructured",
        "pw-identifier-element",
        "pw-reexported",
        "pw-star-explicit",
        "pw-type-star-runtime",
        "pw-type-only-shadow",
        "pw-type-shadowed-import",
        "pw-namespace",
        "pw-namespace-call",
        "pw-nonambiguous-star",
        "pw-object-call-project",
        "pw-object-call-arrow-project",
        "pw-object-call-block-project",
        "pw-object-call-function-project",
        "pw-object-call-expression-project",
        "pw-imported-constant-spread",
        "pw-default-array",
        "pw-default-as",
        "pw-default-arrow",
        "pw-default-arrow-block",
        "pw-default-call",
        "pw-commonjs-default-projects",
        "pw-default-direct-as",
        "pw-default-direct-satisfies",
        "pw-default-direct-type-assertion",
        "pw-default-exported-const",
        "pw-default-function",
        "pw-default-identifier-array",
        "pw-default-identifier-function",
        "pw-default-non-null",
        "pw-default-satisfies",
        "pw-default-type-assertion",
        "pw-default-wrapped-array",
        "pw-default-object",
    ] {
        assert!(
            edge.iter()
                .any(|project| project.name.as_deref() == Some(name)),
            "missing Playwright edge project {name}"
        );
    }
    assert!(!edge
        .iter()
        .any(|project| project.name.as_deref() == Some("pw-namespace-star")));
    assert!(!edge
        .iter()
        .any(|project| project.name.as_deref() == Some("pw-ambiguous-star-a")));
    assert!(!edge
        .iter()
        .any(|project| project.name.as_deref() == Some("pw-ambiguous-star-b")));
    assert!(!edge
        .iter()
        .any(|project| project.name.as_deref() == Some("pw-non-spread-call-array")));
    assert!(!edge
        .iter()
        .any(|project| project.name.as_deref() == Some("pw-non-spread-imported-array")));

    let identifier_path = root.join("playwright.identifier-projects.ts");
    let identifier_source = std::fs::read_to_string(&identifier_path).unwrap();
    let identifier = parse_playwright_fixture(&identifier_source, &identifier_path, &root)
        .unwrap()
        .into_projects(&root, "playwright.identifier-projects.ts");
    assert!(identifier
        .iter()
        .any(|project| project.name.as_deref() == Some("imported")));

    let root_spread_path = root.join("playwright.root-spread.ts");
    let root_spread_source = std::fs::read_to_string(&root_spread_path).unwrap();
    let root_spread = parse_playwright_fixture(&root_spread_source, &root_spread_path, &root)
        .unwrap()
        .into_projects(&root, "playwright.root-spread.ts");
    assert!(root_spread
        .iter()
        .any(|project| project.name.as_deref() == Some("root-spread")));

    let root_namespace_path = root.join("playwright.root-namespace-spread.ts");
    let root_namespace_source = std::fs::read_to_string(&root_namespace_path).unwrap();
    let root_namespace =
        parse_playwright_fixture(&root_namespace_source, &root_namespace_path, &root)
            .unwrap()
            .into_projects(&root, "playwright.root-namespace-spread.ts");
    assert!(root_namespace
        .iter()
        .any(|project| project.name.as_deref() == Some("root-namespace-spread")));
    crate::ast::with_program(
        &root_namespace_path,
        &root_namespace_source,
        |program, _| {
            let bindings = test_config::shared::top_level_object_bindings(program);
            let root_object =
                test_config::shared::default_export_object(program, &bindings).unwrap();
            assert!(test_config::shared::property_expression_deep(
                root_object,
                "testDir",
                &bindings
            )
            .is_some());
        },
    )
    .unwrap();

    let root_define_config_path = root.join("playwright.root-define-config-spread.ts");
    let root_define_config_source = std::fs::read_to_string(&root_define_config_path).unwrap();
    let root_define_config =
        parse_playwright_fixture(&root_define_config_source, &root_define_config_path, &root)
            .unwrap()
            .into_projects(&root, "playwright.root-define-config-spread.ts");
    assert!(root_define_config
        .iter()
        .any(|project| project.name.as_deref() == Some("root-define-config-spread")));

    let root_sourced_reexport_path = root.join("playwright.root-sourced-reexport.ts");
    let root_sourced_reexport_source =
        std::fs::read_to_string(&root_sourced_reexport_path).unwrap();
    let root_sourced_reexport = parse_playwright_fixture(
        &root_sourced_reexport_source,
        &root_sourced_reexport_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-sourced-reexport.ts");
    assert!(root_sourced_reexport
        .iter()
        .any(|project| project.name.as_deref() == Some("root-sourced-reexport")));

    let root_sourced_reexport_nested_path = root.join("playwright.root-sourced-reexport-nested.ts");
    let root_sourced_reexport_nested_source =
        std::fs::read_to_string(&root_sourced_reexport_nested_path).unwrap();
    let root_sourced_reexport_nested = parse_playwright_fixture(
        &root_sourced_reexport_nested_source,
        &root_sourced_reexport_nested_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-sourced-reexport-nested.ts");
    assert!(root_sourced_reexport_nested.iter().any(|project| {
        project.name.as_deref() == Some("pw-root-sourced-reexport-nested")
            && project.include == vec!["pw-root-sourced-reexport-nested/**/*.spec.ts"]
    }));

    let root_default_reexport_path = root.join("playwright.root-default-reexport.ts");
    let root_default_reexport_source =
        std::fs::read_to_string(&root_default_reexport_path).unwrap();
    let root_default_reexport = parse_playwright_fixture(
        &root_default_reexport_source,
        &root_default_reexport_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-default-reexport.ts");
    assert!(root_default_reexport.iter().any(|project| {
        project.name.as_deref() == Some("pw-root-default-reexport")
            && project.include == vec!["pw-root-default-reexport/**/*.spec.ts"]
    }));

    let root_call_spread_path = root.join("playwright.root-call-spread.ts");
    let root_call_spread_source = std::fs::read_to_string(&root_call_spread_path).unwrap();
    let root_call_spread =
        parse_playwright_fixture(&root_call_spread_source, &root_call_spread_path, &root)
            .unwrap()
            .into_projects(&root, "playwright.root-call-spread.ts");
    assert!(root_call_spread.iter().any(|project| {
        project.name.as_deref() == Some("pw-root-call-spread")
            && project.include == vec!["pw-root-call-spread/**/*.spec.ts"]
    }));

    let root_named_member_spread_path = root.join("playwright.root-named-member-spread.ts");
    let root_named_member_spread_source =
        std::fs::read_to_string(&root_named_member_spread_path).unwrap();
    let root_named_member_spread = parse_playwright_fixture(
        &root_named_member_spread_source,
        &root_named_member_spread_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-named-member-spread.ts");
    assert!(root_named_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("pw-root-named-member-spread")
            && project.include == vec!["pw-root-named-member-spread/**/*.spec.ts"]
    }));
    let root_local_member_spread_path = root.join("playwright.root-local-member-spread.ts");
    let root_local_member_spread_source =
        std::fs::read_to_string(&root_local_member_spread_path).unwrap();
    let root_local_member_spread = parse_playwright_fixture(
        &root_local_member_spread_source,
        &root_local_member_spread_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-local-member-spread.ts");
    assert!(root_local_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("pw-root-local-member-spread")
            && project.include == vec!["pw-root-local-member-spread/**/*.spec.ts"]
    }));

    let root_imported_spread_member_path = root.join("playwright.root-imported-spread-member.ts");
    let root_imported_spread_member_source =
        std::fs::read_to_string(&root_imported_spread_member_path).unwrap();
    let root_imported_spread_member = parse_playwright_fixture(
        &root_imported_spread_member_source,
        &root_imported_spread_member_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-imported-spread-member.ts");
    assert!(root_imported_spread_member.iter().any(|project| {
        project.name.as_deref() == Some("pw-root-imported-spread-member")
            && project.include == vec!["pw-root-imported-spread-member/**/*.spec.ts"]
    }));

    let root_import_then_export_path = root.join("playwright.root-import-then-export.ts");
    let root_import_then_export_source =
        std::fs::read_to_string(&root_import_then_export_path).unwrap();
    let root_import_then_export = parse_playwright_fixture(
        &root_import_then_export_source,
        &root_import_then_export_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-import-then-export.ts");
    assert!(root_import_then_export.iter().any(|project| {
        project.name.as_deref() == Some("pw-root-import-then-export")
            && project.include == vec!["pw-root-import-then-export/**/*.spec.ts"]
    }));

    let function_local_path = root.join("playwright.function-local-projects.ts");
    let function_local_source = std::fs::read_to_string(&function_local_path).unwrap();
    let function_local =
        parse_playwright_fixture(&function_local_source, &function_local_path, &root)
            .unwrap()
            .into_projects(&root, "playwright.function-local-projects.ts");
    assert!(function_local
        .iter()
        .any(|project| project.name.as_deref() == Some("pw-function-local-projects")));

    let named_member_path = root.join("playwright.named-member-projects.ts");
    let named_member_source = std::fs::read_to_string(&named_member_path).unwrap();
    let named_member = parse_playwright_fixture(&named_member_source, &named_member_path, &root)
        .unwrap()
        .into_projects(&root, "playwright.named-member-projects.ts");
    assert!(named_member
        .iter()
        .any(|project| project.name.as_deref() == Some("pw-named-member-projects")));

    let named_member_reexport_path = root.join("playwright.named-member-reexport.ts");
    let named_member_reexport_source =
        std::fs::read_to_string(&named_member_reexport_path).unwrap();
    let named_member_reexport = parse_playwright_fixture(
        &named_member_reexport_source,
        &named_member_reexport_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.named-member-reexport.ts");
    assert!(named_member_reexport
        .iter()
        .any(|project| project.name.as_deref() == Some("pw-named-member-projects")));

    let imported_spread_member_map_path = root.join("playwright.imported-spread-member-map.ts");
    let imported_spread_member_map_source =
        std::fs::read_to_string(&imported_spread_member_map_path).unwrap();
    let imported_spread_member_map = parse_playwright_fixture(
        &imported_spread_member_map_source,
        &imported_spread_member_map_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.imported-spread-member-map.ts");
    assert!(imported_spread_member_map.iter().any(|project| {
        project.name.as_deref() == Some("pw-imported-spread-member-map")
            && project.include == vec!["pw-imported-spread-member-map/**/*.spec.ts"]
    }));

    let member_default_reexport_path = root.join("playwright.member-default-reexport.ts");
    let member_default_reexport_source =
        std::fs::read_to_string(&member_default_reexport_path).unwrap();
    let member_default_reexport = parse_playwright_fixture(
        &member_default_reexport_source,
        &member_default_reexport_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.member-default-reexport.ts");
    assert!(member_default_reexport.iter().any(|project| {
        project.name.as_deref() == Some("pw-member-default-reexport")
            && project.include == vec!["pw-member-default-reexport/**/*.spec.ts"]
    }));

    let member_import_then_export_path = root.join("playwright.member-import-then-export.ts");
    let member_import_then_export_source =
        std::fs::read_to_string(&member_import_then_export_path).unwrap();
    let member_import_then_export = parse_playwright_fixture(
        &member_import_then_export_source,
        &member_import_then_export_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.member-import-then-export.ts");
    assert!(member_import_then_export.iter().any(|project| {
        project.name.as_deref() == Some("pw-member-import-then-export")
            && project.include == vec!["pw-member-import-then-export/**/*.spec.ts"]
    }));

    let member_nested_barrel_path = root.join("playwright.member-nested-barrel.ts");
    let member_nested_barrel_source = std::fs::read_to_string(&member_nested_barrel_path).unwrap();
    let member_nested_barrel = parse_playwright_fixture(
        &member_nested_barrel_source,
        &member_nested_barrel_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.member-nested-barrel.ts");
    assert!(member_nested_barrel.iter().any(|project| {
        project.name.as_deref() == Some("pw-member-nested-barrel")
            && project.include == vec!["pw-member-nested-barrel/**/*.spec.ts"]
    }));

    let object_default_reexport_path = root.join("playwright.object-default-reexport.ts");
    let object_default_reexport_source =
        std::fs::read_to_string(&object_default_reexport_path).unwrap();
    let object_default_reexport = parse_playwright_fixture(
        &object_default_reexport_source,
        &object_default_reexport_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.object-default-reexport.ts");
    assert!(object_default_reexport.iter().any(|project| {
        project.name.as_deref() == Some("pw-object-default-reexport")
            && project.include == vec!["pw-object-default-reexport/**/*.spec.ts"]
    }));

    let object_import_then_export_path = root.join("playwright.object-import-then-export.ts");
    let object_import_then_export_source =
        std::fs::read_to_string(&object_import_then_export_path).unwrap();
    let object_import_then_export = parse_playwright_fixture(
        &object_import_then_export_source,
        &object_import_then_export_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.object-import-then-export.ts");
    assert!(object_import_then_export.iter().any(|project| {
        project.name.as_deref() == Some("pw-object-import-then-export")
            && project.include == vec!["pw-object-import-then-export/**/*.spec.ts"]
    }));

    let object_call_local_path = root.join("playwright.object-call-local.ts");
    let object_call_local_source = std::fs::read_to_string(&object_call_local_path).unwrap();
    let object_call_local =
        parse_playwright_fixture(&object_call_local_source, &object_call_local_path, &root)
            .unwrap()
            .into_projects(&root, "playwright.object-call-local.ts");
    assert!(object_call_local.iter().any(|project| {
        project.name.as_deref() == Some("pw-object-call-local")
            && project.include == vec!["pw-object-call-local/**/*.spec.ts"]
    }));

    let object_named_member_spread_path = root.join("playwright.object-named-member-spread.ts");
    let object_named_member_spread_source =
        std::fs::read_to_string(&object_named_member_spread_path).unwrap();
    let object_named_member_spread = parse_playwright_fixture(
        &object_named_member_spread_source,
        &object_named_member_spread_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.object-named-member-spread.ts");
    assert!(object_named_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("pw-object-named-member-spread")
            && project.include == vec!["pw-object-named-member-spread/**/*.spec.ts"]
    }));

    let object_namespace_member_spread_path =
        root.join("playwright.object-namespace-member-spread.ts");
    let object_namespace_member_spread_source =
        std::fs::read_to_string(&object_namespace_member_spread_path).unwrap();
    let object_namespace_member_spread = parse_playwright_fixture(
        &object_namespace_member_spread_source,
        &object_namespace_member_spread_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.object-namespace-member-spread.ts");
    assert!(object_namespace_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("pw-object-namespace-member-spread")
            && project.include == vec!["pw-object-namespace-member-spread/**/*.spec.ts"]
    }));

    let object_sourced_member_spread_path = root.join("playwright.object-sourced-member-spread.ts");
    let object_sourced_member_spread_source =
        std::fs::read_to_string(&object_sourced_member_spread_path).unwrap();
    let object_sourced_member_spread = parse_playwright_fixture(
        &object_sourced_member_spread_source,
        &object_sourced_member_spread_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.object-sourced-member-spread.ts");
    assert!(object_sourced_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("pw-object-sourced-member-spread")
            && project.include == vec!["pw-object-sourced-member-spread/**/*.spec.ts"]
    }));

    let object_import_member_spread_path = root.join("playwright.object-import-member-spread.ts");
    let object_import_member_spread_source =
        std::fs::read_to_string(&object_import_member_spread_path).unwrap();
    let object_import_member_spread = parse_playwright_fixture(
        &object_import_member_spread_source,
        &object_import_member_spread_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.object-import-member-spread.ts");
    assert!(object_import_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("pw-object-import-member-spread")
            && project.include == vec!["pw-object-import-member-spread/**/*.spec.ts"]
    }));

    let object_member_defensive_path = root.join("playwright.object-member-defensive.ts");
    let object_member_defensive_source =
        std::fs::read_to_string(&object_member_defensive_path).unwrap();
    assert!(parse_playwright_fixture(
        &object_member_defensive_source,
        &object_member_defensive_path,
        &root,
    )
    .is_ok());

    let destructured_bound_path = root.join("playwright.destructured-bound-projects.ts");
    let destructured_bound_source = std::fs::read_to_string(&destructured_bound_path).unwrap();
    let destructured_bound =
        parse_playwright_fixture(&destructured_bound_source, &destructured_bound_path, &root)
            .unwrap()
            .into_projects(&root, "playwright.destructured-bound-projects.ts");
    assert!(destructured_bound
        .iter()
        .any(|project| project.name.as_deref() == Some("pw-destructured-bound-projects")));

    let destructured_spread_export_path = root.join("playwright.destructured-spread-export.ts");
    let destructured_spread_export_source =
        std::fs::read_to_string(&destructured_spread_export_path).unwrap();
    let destructured_spread_export = parse_playwright_fixture(
        &destructured_spread_export_source,
        &destructured_spread_export_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.destructured-spread-export.ts");
    assert!(destructured_spread_export.iter().any(|project| {
        project.name.as_deref() == Some("pw-destructured-spread-export")
            && project.include == vec!["pw-destructured-spread-export/**/*.spec.ts"]
    }));

    let imported_nested_spread_path = root.join("playwright.imported-nested-spread.ts");
    let imported_nested_spread_source =
        std::fs::read_to_string(&imported_nested_spread_path).unwrap();
    let imported_nested_spread = parse_playwright_fixture(
        &imported_nested_spread_source,
        &imported_nested_spread_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.imported-nested-spread.ts");
    assert!(imported_nested_spread.iter().any(|project| {
        project.name.as_deref() == Some("pw-imported-nested-spread")
            && project.include == vec!["pw-imported-nested-spread/**/*.spec.ts"]
            && project.exclude == vec!["pw-imported-nested-spread/**/*.skip.ts"]
    }));

    let local_member_spread_path = root.join("playwright.local-member-spread.ts");
    let local_member_spread_source = std::fs::read_to_string(&local_member_spread_path).unwrap();
    let local_member_spread = parse_playwright_fixture(
        &local_member_spread_source,
        &local_member_spread_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.local-member-spread.ts");
    assert!(local_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("pw-local-member-spread")
            && project.include == vec!["pw-local-member-spread/**/*.spec.ts"]
            && project.exclude == vec!["pw-local-member-spread/**/*.skip.ts"]
    }));

    let empty_star_path = root.join("playwright.empty-star.ts");
    let empty_star_source = std::fs::read_to_string(&empty_star_path).unwrap();
    let empty_star = parse_playwright_fixture(&empty_star_source, &empty_star_path, &root)
        .unwrap()
        .into_projects(&root, "playwright.empty-star.ts");
    assert!(!empty_star
        .iter()
        .any(|project| project.name.as_deref() == Some("pw-empty-star-runtime")));

    let root_spread_empty_path = root.join("playwright.root-spread-empty.ts");
    let root_spread_empty_source = std::fs::read_to_string(&root_spread_empty_path).unwrap();
    let root_spread_empty =
        parse_playwright_fixture(&root_spread_empty_source, &root_spread_empty_path, &root)
            .unwrap()
            .into_projects(&root, "playwright.root-spread-empty.ts");
    assert!(root_spread_empty
        .iter()
        .any(|project| project.name.as_deref() == Some("ignored-specifier-config")));

    let root_spread_defensive_path = root.join("playwright.root-spread-defensive.ts");
    let root_spread_defensive_source =
        std::fs::read_to_string(&root_spread_defensive_path).unwrap();
    parse_playwright_fixture(
        &root_spread_defensive_source,
        &root_spread_defensive_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-spread-defensive.ts");
    for file in [
        "playwright.root-spread-default-as.ts",
        "playwright.root-spread-default-satisfies.ts",
        "playwright.root-spread-default-type-assertion.ts",
    ] {
        let path = root.join(file);
        let source = std::fs::read_to_string(&path).unwrap();
        crate::ast::with_program(&path, &source, |program, _| {
            let bindings = test_config::shared::top_level_object_bindings(program);
            assert!(test_config::shared::default_export_object(program, &bindings).is_some());
        })
        .unwrap();
    }

    let root_imported_path = root.join("playwright.root-imported-config.ts");
    let root_imported_source = std::fs::read_to_string(&root_imported_path).unwrap();
    let root_imported = parse_playwright_fixture(&root_imported_source, &root_imported_path, &root)
        .unwrap()
        .into_projects(&root, "playwright.root-imported-config.ts");
    assert!(root_imported.iter().any(|project| {
        project.name.as_deref() == Some("root-imported-config")
            && project.include == vec!["root-imported-defaults/**/*.shared.spec.ts"]
    }));

    let root_spread_order_path = root.join("playwright.root-spread-order.ts");
    let root_spread_order_source = std::fs::read_to_string(&root_spread_order_path).unwrap();
    let root_spread_order =
        parse_playwright_fixture(&root_spread_order_source, &root_spread_order_path, &root)
            .unwrap()
            .into_projects(&root, "playwright.root-spread-order.ts");
    assert!(root_spread_order.iter().any(|project| {
        project.name.as_deref() == Some("root-spread-order-shared")
            && project.include == vec!["root-spread-order/**/*.shared.spec.ts"]
    }));
    assert!(!root_spread_order
        .iter()
        .any(|project| project.name.as_deref() == Some("root-spread-order-local")));

    let root_named_imported_path = root.join("playwright.root-named-imported-config.ts");
    let root_named_imported_source = std::fs::read_to_string(&root_named_imported_path).unwrap();
    let root_named_imported = parse_playwright_fixture(
        &root_named_imported_source,
        &root_named_imported_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-named-imported-config.ts");
    assert!(root_named_imported
        .iter()
        .any(|project| project.name.as_deref() == Some("root-named-imported-config")));

    let root_alias_imported_path = root.join("playwright.root-local-alias-imported-config.ts");
    let root_alias_imported_source = std::fs::read_to_string(&root_alias_imported_path).unwrap();
    let root_alias_imported = parse_playwright_fixture(
        &root_alias_imported_source,
        &root_alias_imported_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-local-alias-imported-config.ts");
    assert!(root_alias_imported.iter().any(|project| {
        project.name.as_deref() == Some("root-local-alias-imported-config")
            && project.include == vec!["root-local-alias-imported-config/**/*.spec.ts"]
    }));

    let empty_match_path = root.join("playwright.empty-test-match.ts");
    let empty_match_source = std::fs::read_to_string(&empty_match_path).unwrap();
    assert!(parse_playwright_fixture(&empty_match_source, &empty_match_path, &root).is_err());

    let root_call_spread_local_path = root.join("playwright.root-call-spread-local.ts");
    let root_call_spread_local_source =
        std::fs::read_to_string(&root_call_spread_local_path).unwrap();
    let root_call_spread_local = parse_playwright_fixture(
        &root_call_spread_local_source,
        &root_call_spread_local_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-call-spread-local.ts");
    assert!(root_call_spread_local.iter().any(|project| {
        project.name.as_deref() == Some("pw-root-call-local")
    }));

    let root_member_import_then_export_path =
        root.join("playwright.root-member-import-then-export.ts");
    let root_member_import_then_export_source =
        std::fs::read_to_string(&root_member_import_then_export_path).unwrap();
    let root_member_import_then_export = parse_playwright_fixture(
        &root_member_import_then_export_source,
        &root_member_import_then_export_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-member-import-then-export.ts");
    assert!(root_member_import_then_export.iter().any(|project| {
        project.name.as_deref() == Some("pw-root-member-import-then-export")
    }));

    let object_call_import_path = root.join("playwright.object-call-import.ts");
    let object_call_import_source = std::fs::read_to_string(&object_call_import_path).unwrap();
    let object_call_import = parse_playwright_fixture(
        &object_call_import_source,
        &object_call_import_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.object-call-import.ts");
    assert!(object_call_import.iter().any(|project| {
        project.name.as_deref() == Some("pw-object-call-import")
            && project.include == vec!["pw-object-call-import/**/*.spec.ts"]
    }));

    let root_call_import_path = root.join("playwright.root-call-import.ts");
    let root_call_import_source = std::fs::read_to_string(&root_call_import_path).unwrap();
    let root_call_import = parse_playwright_fixture(
        &root_call_import_source,
        &root_call_import_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-call-import.ts");
    assert!(root_call_import.iter().any(|project| {
        project.name.as_deref() == Some("pw-root-call-import")
            && project.include == vec!["pw-root-call-import/**/*.spec.ts"]
    }));

    let member_namespace_star_path = root.join("playwright.member-namespace-star.ts");
    let member_namespace_star_source =
        std::fs::read_to_string(&member_namespace_star_path).unwrap();
    let member_namespace_star = parse_playwright_fixture(
        &member_namespace_star_source,
        &member_namespace_star_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.member-namespace-star.ts");
    assert!(member_namespace_star.iter().any(|project| {
        project.name.as_deref() == Some("pw-member-namespace-star")
            && project.include == vec!["pw-member-namespace-star/**/*.spec.ts"]
    }));

    let root_star_barrel_import_path = root.join("playwright.root-star-barrel-import.ts");
    let root_star_barrel_import_source =
        std::fs::read_to_string(&root_star_barrel_import_path).unwrap();
    let root_star_barrel_import = parse_playwright_fixture(
        &root_star_barrel_import_source,
        &root_star_barrel_import_path,
        &root,
    )
    .unwrap()
    .into_projects(&root, "playwright.root-star-barrel-import.ts");
    assert!(root_star_barrel_import.iter().any(|project| {
        project.name.as_deref() == Some("pw-root-star-barrel")
            && project.include == vec!["pw-root-star-barrel/**/*.spec.ts"]
    }));
}

#[test]
fn vitest_config_parser_covers_root_and_nested_projects() {
    let root = fixture("coverage");
    let object_path = root.join("vitest.object.mts");
    let object_source = std::fs::read_to_string(&object_path).unwrap();
    let object_projects = parse_vitest_fixture(&object_source, &object_path, &root).unwrap();
    assert_eq!(object_projects[0].name.as_deref(), Some("root-vitest"));

    let projects_path = root.join("vitest.projects.mts");
    let projects_source = std::fs::read_to_string(&projects_path).unwrap();
    let projects = parse_vitest_fixture(&projects_source, &projects_path, &root).unwrap();
    assert!(projects
        .iter()
        .any(|project| project.name.as_deref() == Some("nested")));
    assert!(projects
        .iter()
        .any(|project| project.name.as_deref() == Some("root")));

    let empty_path = root.join("vitest.empty.mts");
    let empty_source = std::fs::read_to_string(&empty_path).unwrap();
    assert!(parse_vitest_fixture(&empty_source, &empty_path, &root)
        .unwrap()
        .is_empty());

    let defaults_path = root.join("vitest.defaults.mts");
    let defaults_source = std::fs::read_to_string(&defaults_path).unwrap();
    let defaults = parse_vitest_fixture(&defaults_source, &defaults_path, &root).unwrap();
    assert!(defaults[0]
        .include
        .iter()
        .any(|glob| glob.contains("__tests__")));

    let dynamic_path = root.join("vitest.dynamic.mts");
    let dynamic_source = std::fs::read_to_string(&dynamic_path).unwrap();
    let dynamic = parse_vitest_fixture(&dynamic_source, &dynamic_path, &root).unwrap();
    assert!(dynamic.iter().any(|project| {
        project.name.as_deref() == Some("web")
            && project.include == vec!["web/**/*.test.ts"]
            && project.exclude == vec!["web/**/*.skip.ts"]
    }));
    assert!(dynamic.iter().any(|project| {
        project.name.as_deref() == Some("local") && project.include == vec!["local/**/*.test.ts"]
    }));
    assert!(dynamic.iter().any(|project| {
        project.name.as_deref() == Some("api") && project.include == vec!["api/**/*.test.ts"]
    }));
    assert!(dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("composed")));
    assert!(dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("default-import")));
    assert!(dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("default-arrow")));
    assert!(dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("namespace")));
    assert!(dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("same-name-import")));
    assert!(dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("reexported")));
    assert!(dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("alias-default")));
    assert!(dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("default-call")));
    assert!(!dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("default-call-arg")));
    assert!(dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("default-function")));
    assert!(dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("default-array")));
    assert!(dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("local-exported")));
    assert!(dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("local-exported-function")));
    assert!(dynamic
        .iter()
        .any(|project| project.name.as_deref() == Some("namespace-array")));
    assert!(dynamic.iter().any(|project| {
        project.name.as_deref() == Some("spread-object")
            && project.include == vec!["spread-object/**/*.test.ts"]
            && project.exclude == vec!["spread-object/**/*.skip.ts"]
    }));

    let imported_test_spread_path = root.join("vitest.imported-test-spread.mts");
    let imported_test_spread_source = std::fs::read_to_string(&imported_test_spread_path).unwrap();
    let imported_test_spread = parse_vitest_fixture(
        &imported_test_spread_source,
        &imported_test_spread_path,
        &root,
    )
    .unwrap();
    assert!(imported_test_spread.iter().any(|project| {
        project.name.as_deref() == Some("imported-test-spread")
            && project.include == vec!["imported-test-spread/**/*.test.ts"]
    }));

    let commonjs_projects_path = root.join("vitest.commonjs-projects.mts");
    let commonjs_projects_source = std::fs::read_to_string(&commonjs_projects_path).unwrap();
    let commonjs_projects =
        parse_vitest_fixture(&commonjs_projects_source, &commonjs_projects_path, &root).unwrap();
    assert!(commonjs_projects.iter().any(|project| {
        project.name.as_deref() == Some("vitest-commonjs-default-projects")
            && project.include == vec!["vitest-commonjs-default-projects/**/*.test.ts"]
    }));

    let function_local_path = root.join("vitest.function-local-projects.mts");
    let function_local_source = std::fs::read_to_string(&function_local_path).unwrap();
    let function_local =
        parse_vitest_fixture(&function_local_source, &function_local_path, &root).unwrap();
    assert!(function_local.iter().any(|project| {
        project.name.as_deref() == Some("vitest-function-local-projects")
            && project.include == vec!["vitest-function-local-projects/**/*.test.ts"]
    }));

    let root_spread_order_path = root.join("vitest.root-spread-order.mts");
    let root_spread_order_source = std::fs::read_to_string(&root_spread_order_path).unwrap();
    let root_spread_order =
        parse_vitest_fixture(&root_spread_order_source, &root_spread_order_path, &root).unwrap();
    assert!(root_spread_order.iter().any(|project| {
        project.name.as_deref() == Some("root-spread-order-shared")
            && project.include == vec!["root-spread-order/**/*.test.ts"]
    }));
    assert!(!root_spread_order
        .iter()
        .any(|project| project.name.as_deref() == Some("root-spread-order-local")));

    let root_spread_override_path = root.join("vitest.root-spread-overrides-test.mts");
    let root_spread_override_source = std::fs::read_to_string(&root_spread_override_path).unwrap();
    let root_spread_override = parse_vitest_fixture(
        &root_spread_override_source,
        &root_spread_override_path,
        &root,
    )
    .unwrap();
    assert!(root_spread_override.iter().any(|project| {
        project.name.as_deref() == Some("root-spread-overrides-test")
            && project.include == vec!["root-spread-overrides-test/**/*.test.ts"]
    }));
    assert!(!root_spread_override
        .iter()
        .any(|project| project.name.as_deref() == Some("root-spread-overrides-local")));

    let root_test_override_clears_path = root.join("vitest.root-test-override-clears.mts");
    let root_test_override_clears_source =
        std::fs::read_to_string(&root_test_override_clears_path).unwrap();
    let root_test_override_clears = parse_vitest_fixture(
        &root_test_override_clears_source,
        &root_test_override_clears_path,
        &root,
    )
    .unwrap();
    assert!(!root_test_override_clears
        .iter()
        .any(|project| project.name.as_deref() == Some("vitest-stale-root-spread-project")));

    let root_namespace_path = root.join("vitest.root-namespace-spread.mts");
    let root_namespace_source = std::fs::read_to_string(&root_namespace_path).unwrap();
    let root_namespace =
        parse_vitest_fixture(&root_namespace_source, &root_namespace_path, &root).unwrap();
    assert!(root_namespace.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-namespace-spread")
            && project.include == vec!["vitest-root-namespace-spread/**/*.test.ts"]
    }));

    let test_namespace_path = root.join("vitest.test-namespace-spread.mts");
    let test_namespace_source = std::fs::read_to_string(&test_namespace_path).unwrap();
    let test_namespace =
        parse_vitest_fixture(&test_namespace_source, &test_namespace_path, &root).unwrap();
    assert!(test_namespace.iter().any(|project| {
        project.name.as_deref() == Some("vitest-test-namespace-spread")
            && project.include == vec!["vitest-test-namespace-spread/**/*.test.ts"]
    }));

    let root_define_config_path = root.join("vitest.root-define-config-spread.mts");
    let root_define_config_source = std::fs::read_to_string(&root_define_config_path).unwrap();
    let root_define_config =
        parse_vitest_fixture(&root_define_config_source, &root_define_config_path, &root).unwrap();
    assert!(root_define_config.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-define-config-spread")
            && project.include == vec!["vitest-root-define-config-spread/**/*.test.ts"]
    }));

    let root_sourced_reexport_path = root.join("vitest.root-sourced-reexport.mts");
    let root_sourced_reexport_source =
        std::fs::read_to_string(&root_sourced_reexport_path).unwrap();
    let root_sourced_reexport = parse_vitest_fixture(
        &root_sourced_reexport_source,
        &root_sourced_reexport_path,
        &root,
    )
    .unwrap();
    assert!(root_sourced_reexport.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-sourced-reexport")
            && project.include == vec!["vitest-root-sourced-reexport/**/*.test.ts"]
    }));

    let root_sourced_reexport_nested_path = root.join("vitest.root-sourced-reexport-nested.mts");
    let root_sourced_reexport_nested_source =
        std::fs::read_to_string(&root_sourced_reexport_nested_path).unwrap();
    let root_sourced_reexport_nested = parse_vitest_fixture(
        &root_sourced_reexport_nested_source,
        &root_sourced_reexport_nested_path,
        &root,
    )
    .unwrap();
    assert!(root_sourced_reexport_nested.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-sourced-reexport-nested")
            && project.include == vec!["vitest-root-sourced-reexport-nested/**/*.test.ts"]
    }));

    let root_default_reexport_path = root.join("vitest.root-default-reexport.mts");
    let root_default_reexport_source =
        std::fs::read_to_string(&root_default_reexport_path).unwrap();
    let root_default_reexport = parse_vitest_fixture(
        &root_default_reexport_source,
        &root_default_reexport_path,
        &root,
    )
    .unwrap();
    assert!(root_default_reexport.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-default-reexport")
            && project.include == vec!["vitest-root-default-reexport/**/*.test.ts"]
    }));

    let root_spread_options_path = root.join("vitest.root-spread-options.mts");
    let root_spread_options_source = std::fs::read_to_string(&root_spread_options_path).unwrap();
    let root_spread_options = parse_vitest_fixture(
        &root_spread_options_source,
        &root_spread_options_path,
        &root,
    )
    .unwrap();
    assert!(root_spread_options.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-spread-options")
            && project.include == vec!["vitest-root-spread-options/**/*.test.ts"]
            && project.exclude == vec!["vitest-root-spread-options/**/*.skip.ts"]
    }));
    assert!(!root_spread_options.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-spread-options")
            && project
                .include
                .contains(&"vitest-root-spread-options-stale/**/*.test.ts".to_string())
    }));

    let root_call_spread_path = root.join("vitest.root-call-spread.mts");
    let root_call_spread_source = std::fs::read_to_string(&root_call_spread_path).unwrap();
    let root_call_spread =
        parse_vitest_fixture(&root_call_spread_source, &root_call_spread_path, &root).unwrap();
    assert!(root_call_spread.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-call-spread")
            && project.include == vec!["vitest-root-call-spread/**/*.test.ts"]
    }));
    let test_call_spread_path = root.join("vitest.test-call-spread.mts");
    let test_call_spread_source = std::fs::read_to_string(&test_call_spread_path).unwrap();
    let test_call_spread =
        parse_vitest_fixture(&test_call_spread_source, &test_call_spread_path, &root).unwrap();
    assert!(test_call_spread.iter().any(|project| {
        project.name.as_deref() == Some("vitest-test-call-spread")
            && project.include == vec!["vitest-test-call-spread/**/*.test.ts"]
    }));

    let root_named_member_spread_path = root.join("vitest.root-named-member-spread.mts");
    let root_named_member_spread_source =
        std::fs::read_to_string(&root_named_member_spread_path).unwrap();
    let root_named_member_spread = parse_vitest_fixture(
        &root_named_member_spread_source,
        &root_named_member_spread_path,
        &root,
    )
    .unwrap();
    assert!(root_named_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-named-member-spread")
            && project.include == vec!["vitest-root-named-member-spread/**/*.test.ts"]
    }));

    let root_local_member_spread_path = root.join("vitest.root-local-member-spread.mts");
    let root_local_member_spread_source =
        std::fs::read_to_string(&root_local_member_spread_path).unwrap();
    let root_local_member_spread = parse_vitest_fixture(
        &root_local_member_spread_source,
        &root_local_member_spread_path,
        &root,
    )
    .unwrap();
    assert!(root_local_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-local-member-spread")
            && project.include == vec!["vitest-root-local-member-spread/**/*.test.ts"]
    }));

    let root_imported_spread_member_path = root.join("vitest.root-imported-spread-member.mts");
    let root_imported_spread_member_source =
        std::fs::read_to_string(&root_imported_spread_member_path).unwrap();
    let root_imported_spread_member = parse_vitest_fixture(
        &root_imported_spread_member_source,
        &root_imported_spread_member_path,
        &root,
    )
    .unwrap();
    assert!(root_imported_spread_member.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-imported-spread-member")
            && project.include == vec!["vitest-root-imported-spread-member/**/*.test.ts"]
    }));

    let root_import_then_export_path = root.join("vitest.root-import-then-export.mts");
    let root_import_then_export_source =
        std::fs::read_to_string(&root_import_then_export_path).unwrap();
    let root_import_then_export = parse_vitest_fixture(
        &root_import_then_export_source,
        &root_import_then_export_path,
        &root,
    )
    .unwrap();
    assert!(root_import_then_export.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-import-then-export")
            && project.include == vec!["vitest-root-import-then-export/**/*.test.ts"]
    }));

    let test_named_member_spread_path = root.join("vitest.test-named-member-spread.mts");
    let test_named_member_spread_source =
        std::fs::read_to_string(&test_named_member_spread_path).unwrap();
    let test_named_member_spread = parse_vitest_fixture(
        &test_named_member_spread_source,
        &test_named_member_spread_path,
        &root,
    )
    .unwrap();
    assert!(test_named_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("vitest-test-named-member-spread")
            && project.include == vec!["vitest-test-named-member-spread/**/*.test.ts"]
    }));

    let test_local_member_spread_path = root.join("vitest.test-local-member-spread.mts");
    let test_local_member_spread_source =
        std::fs::read_to_string(&test_local_member_spread_path).unwrap();
    let test_local_member_spread = parse_vitest_fixture(
        &test_local_member_spread_source,
        &test_local_member_spread_path,
        &root,
    )
    .unwrap();
    assert!(test_local_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("vitest-test-local-member-spread")
            && project.include == vec!["vitest-test-local-member-spread/**/*.test.ts"]
    }));

    let member_nested_barrel_path = root.join("vitest.member-nested-barrel.mts");
    let member_nested_barrel_source = std::fs::read_to_string(&member_nested_barrel_path).unwrap();
    let member_nested_barrel = parse_vitest_fixture(
        &member_nested_barrel_source,
        &member_nested_barrel_path,
        &root,
    )
    .unwrap();
    assert!(member_nested_barrel.iter().any(|project| {
        project.name.as_deref() == Some("vitest-member-nested-barrel")
            && project.include == vec!["vitest-member-nested-barrel/**/*.test.ts"]
    }));

    let member_default_reexport_path = root.join("vitest.member-default-reexport.mts");
    let member_default_reexport_source =
        std::fs::read_to_string(&member_default_reexport_path).unwrap();
    let member_default_reexport = parse_vitest_fixture(
        &member_default_reexport_source,
        &member_default_reexport_path,
        &root,
    )
    .unwrap();
    assert!(member_default_reexport.iter().any(|project| {
        project.name.as_deref() == Some("vitest-member-default-reexport")
            && project.include == vec!["vitest-member-default-reexport/**/*.test.ts"]
    }));

    let member_import_then_export_path = root.join("vitest.member-import-then-export.mts");
    let member_import_then_export_source =
        std::fs::read_to_string(&member_import_then_export_path).unwrap();
    let member_import_then_export = parse_vitest_fixture(
        &member_import_then_export_source,
        &member_import_then_export_path,
        &root,
    )
    .unwrap();
    assert!(member_import_then_export.iter().any(|project| {
        project.name.as_deref() == Some("vitest-member-import-then-export")
            && project.include == vec!["vitest-member-import-then-export/**/*.test.ts"]
    }));

    let object_default_reexport_path = root.join("vitest.object-default-reexport.mts");
    let object_default_reexport_source =
        std::fs::read_to_string(&object_default_reexport_path).unwrap();
    let object_default_reexport = parse_vitest_fixture(
        &object_default_reexport_source,
        &object_default_reexport_path,
        &root,
    )
    .unwrap();
    assert!(object_default_reexport.iter().any(|project| {
        project.name.as_deref() == Some("vitest-object-default-reexport")
            && project.include == vec!["vitest-object-default-reexport/**/*.test.ts"]
    }));

    let object_nested_barrel_path = root.join("vitest.object-nested-barrel.mts");
    let object_nested_barrel_source = std::fs::read_to_string(&object_nested_barrel_path).unwrap();
    let object_nested_barrel = parse_vitest_fixture(
        &object_nested_barrel_source,
        &object_nested_barrel_path,
        &root,
    )
    .unwrap();
    assert!(object_nested_barrel.iter().any(|project| {
        project.name.as_deref() == Some("vitest-object-nested-barrel")
            && project.include == vec!["vitest-object-nested-barrel/**/*.test.ts"]
    }));

    let object_import_then_export_path = root.join("vitest.object-import-then-export.mts");
    let object_import_then_export_source =
        std::fs::read_to_string(&object_import_then_export_path).unwrap();
    let object_import_then_export = parse_vitest_fixture(
        &object_import_then_export_source,
        &object_import_then_export_path,
        &root,
    )
    .unwrap();
    assert!(object_import_then_export.iter().any(|project| {
        project.name.as_deref() == Some("vitest-object-import-then-export")
            && project.include == vec!["vitest-object-import-then-export/**/*.test.ts"]
    }));

    let object_call_local_path = root.join("vitest.object-call-local.mts");
    let object_call_local_source = std::fs::read_to_string(&object_call_local_path).unwrap();
    let object_call_local =
        parse_vitest_fixture(&object_call_local_source, &object_call_local_path, &root).unwrap();
    assert!(object_call_local.iter().any(|project| {
        project.name.as_deref() == Some("vitest-object-call-local")
            && project.include == vec!["vitest-object-call-local/**/*.test.ts"]
    }));

    let object_named_member_spread_path = root.join("vitest.object-named-member-spread.mts");
    let object_named_member_spread_source =
        std::fs::read_to_string(&object_named_member_spread_path).unwrap();
    let object_named_member_spread = parse_vitest_fixture(
        &object_named_member_spread_source,
        &object_named_member_spread_path,
        &root,
    )
    .unwrap();
    assert!(object_named_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("vitest-object-named-member-spread")
            && project.include == vec!["packages/vitest-object-named-member-spread/**/*.test.ts"]
    }));

    let object_sourced_member_spread_path = root.join("vitest.object-sourced-member-spread.mts");
    let object_sourced_member_spread_source =
        std::fs::read_to_string(&object_sourced_member_spread_path).unwrap();
    let object_sourced_member_spread = parse_vitest_fixture(
        &object_sourced_member_spread_source,
        &object_sourced_member_spread_path,
        &root,
    )
    .unwrap();
    assert!(object_sourced_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("vitest-object-sourced-member-spread")
            && project.include == vec!["vitest-object-sourced-member-spread/**/*.test.ts"]
    }));

    let object_import_member_spread_path = root.join("vitest.object-import-member-spread.mts");
    let object_import_member_spread_source =
        std::fs::read_to_string(&object_import_member_spread_path).unwrap();
    let object_import_member_spread = parse_vitest_fixture(
        &object_import_member_spread_source,
        &object_import_member_spread_path,
        &root,
    )
    .unwrap();
    assert!(object_import_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("vitest-object-import-member-spread")
            && project.include == vec!["vitest-object-import-member-spread/**/*.test.ts"]
    }));

    let object_member_defensive_path = root.join("vitest.object-member-defensive.mts");
    let object_member_defensive_source =
        std::fs::read_to_string(&object_member_defensive_path).unwrap();
    assert!(parse_vitest_fixture(
        &object_member_defensive_source,
        &object_member_defensive_path,
        &root
    )
    .is_ok());

    let test_sourced_reexport_path = root.join("vitest.test-sourced-reexport.mts");
    let test_sourced_reexport_source =
        std::fs::read_to_string(&test_sourced_reexport_path).unwrap();
    let test_sourced_reexport = parse_vitest_fixture(
        &test_sourced_reexport_source,
        &test_sourced_reexport_path,
        &root,
    )
    .unwrap();
    assert!(test_sourced_reexport.iter().any(|project| {
        project.name.as_deref() == Some("vitest-test-sourced-reexport")
            && project.include == vec!["vitest-test-sourced-reexport/**/*.test.ts"]
    }));
    assert!(!test_sourced_reexport
        .iter()
        .any(|project| { project.name.as_deref() == Some("vitest-nested-test-sourced-reexport") }));

    let imported_test_object_path = root.join("vitest.imported-test-object.mts");
    let imported_test_object_source = std::fs::read_to_string(&imported_test_object_path).unwrap();
    let imported_test_object = parse_vitest_fixture(
        &imported_test_object_source,
        &imported_test_object_path,
        &root,
    )
    .unwrap();
    assert!(imported_test_object.iter().any(|project| {
        project.name.as_deref() == Some("vitest-imported-test-object")
            && project.include == vec!["vitest-imported-test-object/**/*.test.ts"]
    }));

    let imported_project_test_block_path = root.join("vitest.imported-project-test-block.mts");
    let imported_project_test_block_source =
        std::fs::read_to_string(&imported_project_test_block_path).unwrap();
    let imported_project_test_block = parse_vitest_fixture(
        &imported_project_test_block_source,
        &imported_project_test_block_path,
        &root,
    )
    .unwrap();
    assert!(imported_project_test_block.iter().any(|project| {
        project.name.as_deref() == Some("vitest-imported-project-test-block")
            && project.include == vec!["vitest-imported-project-test-block/**/*.test.ts"]
            && project.exclude == vec!["vitest-imported-project-test-block/**/*.skip.ts"]
    }));

    let root_top_level_projects_path = root.join("vitest.root-top-level-projects-ignored.mts");
    let root_top_level_projects_source =
        std::fs::read_to_string(&root_top_level_projects_path).unwrap();
    let root_top_level_projects = parse_vitest_fixture(
        &root_top_level_projects_source,
        &root_top_level_projects_path,
        &root,
    )
    .unwrap();
    assert!(root_top_level_projects.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-test-projects")
            && project.include == vec!["vitest-root-test-projects/**/*.test.ts"]
    }));
    assert!(!root_top_level_projects
        .iter()
        .any(|project| project.name.as_deref() == Some("vitest-root-top-level-projects")));

    let root_imported_path = root.join("vitest.root-imported-config.mts");
    let root_imported_source = std::fs::read_to_string(&root_imported_path).unwrap();
    let root_imported =
        parse_vitest_fixture(&root_imported_source, &root_imported_path, &root).unwrap();
    assert!(root_imported.iter().any(|project| {
        project.name.as_deref() == Some("root-imported-test-projects")
            && project.include == vec!["root-imported-test-projects/**/*.test.ts"]
    }));

    let root_alias_imported_path = root.join("vitest.root-alias-imported-config.mts");
    let root_alias_imported_source = std::fs::read_to_string(&root_alias_imported_path).unwrap();
    let root_alias_imported = parse_vitest_fixture(
        &root_alias_imported_source,
        &root_alias_imported_path,
        &root,
    )
    .unwrap();
    assert!(root_alias_imported.iter().any(|project| {
        project.name.as_deref() == Some("root-alias-imported-test-projects")
            && project.include == vec!["root-alias-imported-test-projects/**/*.test.ts"]
    }));

    let named_imported_test_spread_path = root.join("vitest.named-imported-test-spread.mts");
    let named_imported_test_spread_source =
        std::fs::read_to_string(&named_imported_test_spread_path).unwrap();
    let named_imported_test_spread = parse_vitest_fixture(
        &named_imported_test_spread_source,
        &named_imported_test_spread_path,
        &root,
    )
    .unwrap();
    assert!(named_imported_test_spread.iter().any(|project| {
        project.name.as_deref() == Some("named-imported-test-spread")
            && project.include == vec!["named-imported-test-spread/**/*.test.ts"]
    }));

    let named_member_path = root.join("vitest.named-member-projects.mts");
    let named_member_source = std::fs::read_to_string(&named_member_path).unwrap();
    let named_member =
        parse_vitest_fixture(&named_member_source, &named_member_path, &root).unwrap();
    assert!(named_member.iter().any(|project| {
        project.name.as_deref() == Some("vitest-named-member-projects")
            && project.include == vec!["vitest-named-member-projects/**/*.test.ts"]
    }));

    let named_member_reexport_path = root.join("vitest.named-member-reexport.mts");
    let named_member_reexport_source =
        std::fs::read_to_string(&named_member_reexport_path).unwrap();
    let named_member_reexport = parse_vitest_fixture(
        &named_member_reexport_source,
        &named_member_reexport_path,
        &root,
    )
    .unwrap();
    assert!(named_member_reexport.iter().any(|project| {
        project.name.as_deref() == Some("vitest-named-member-projects")
            && project.include == vec!["vitest-named-member-projects/**/*.test.ts"]
    }));

    let imported_spread_member_map_path = root.join("vitest.imported-spread-member-map.mts");
    let imported_spread_member_map_source =
        std::fs::read_to_string(&imported_spread_member_map_path).unwrap();
    let imported_spread_member_map = parse_vitest_fixture(
        &imported_spread_member_map_source,
        &imported_spread_member_map_path,
        &root,
    )
    .unwrap();
    assert!(imported_spread_member_map.iter().any(|project| {
        project.name.as_deref() == Some("vitest-imported-spread-member-map")
            && project.include == vec!["vitest-imported-spread-member-map/**/*.test.ts"]
    }));

    let destructured_bound_path = root.join("vitest.destructured-bound-projects.mts");
    let destructured_bound_source = std::fs::read_to_string(&destructured_bound_path).unwrap();
    let destructured_bound =
        parse_vitest_fixture(&destructured_bound_source, &destructured_bound_path, &root).unwrap();
    assert!(destructured_bound.iter().any(|project| {
        project.name.as_deref() == Some("vitest-destructured-bound-projects")
            && project.include == vec!["vitest-destructured-bound-projects/**/*.test.ts"]
    }));

    let destructured_spread_export_path = root.join("vitest.destructured-spread-export.mts");
    let destructured_spread_export_source =
        std::fs::read_to_string(&destructured_spread_export_path).unwrap();
    let destructured_spread_export = parse_vitest_fixture(
        &destructured_spread_export_source,
        &destructured_spread_export_path,
        &root,
    )
    .unwrap();
    assert!(destructured_spread_export.iter().any(|project| {
        project.name.as_deref() == Some("vitest-destructured-spread-export")
            && project.include == vec!["vitest-destructured-spread-export/**/*.test.ts"]
    }));

    let empty_star_path = root.join("vitest.empty-star.mts");
    let empty_star_source = std::fs::read_to_string(&empty_star_path).unwrap();
    let empty_star = parse_vitest_fixture(&empty_star_source, &empty_star_path, &root).unwrap();
    assert!(!empty_star
        .iter()
        .any(|project| project.name.as_deref() == Some("vitest-empty-star-runtime")));

    let root_spread_empty_path = root.join("vitest.root-spread-empty.mts");
    let root_spread_empty_source = std::fs::read_to_string(&root_spread_empty_path).unwrap();
    let root_spread_empty =
        parse_vitest_fixture(&root_spread_empty_source, &root_spread_empty_path, &root).unwrap();
    assert!(root_spread_empty.iter().any(|project| {
        project.name.as_deref() == Some("ignored-specifier-config")
            && project.include == vec!["ignored-specifier-config/**/*.test.ts"]
    }));

    let root_spread_defensive_path = root.join("vitest.root-spread-defensive.mts");
    let root_spread_defensive_source =
        std::fs::read_to_string(&root_spread_defensive_path).unwrap();
    parse_vitest_fixture(
        &root_spread_defensive_source,
        &root_spread_defensive_path,
        &root,
    )
    .unwrap();

    let identifier_path = root.join("vitest.identifier.mts");
    let identifier_source = std::fs::read_to_string(&identifier_path).unwrap();
    let identifier = parse_vitest_fixture(&identifier_source, &identifier_path, &root).unwrap();
    assert!(identifier
        .iter()
        .any(|project| project.name.as_deref() == Some("reexported")));
    assert!(identifier
        .iter()
        .any(|project| project.name.as_deref() == Some("local-identifier-projects")));
    assert!(identifier
        .iter()
        .any(|project| project.name.as_deref() == Some("default-object")));

    let edge_path = root.join("vitest.edge.mts");
    let edge_source = std::fs::read_to_string(&edge_path).unwrap();
    let edge = parse_vitest_fixture(&edge_source, &edge_path, &root).unwrap();
    for name in [
        "parenthesized",
        "wrapped-helper",
        "vitest-type-star-runtime",
        "function-expression",
        "block-arrow",
        "top-level-function",
        "named-var",
        "named-function",
        "overloaded-function",
        "local-alias",
        "local-function",
        "reexported",
        "destructured-export",
        "typed-reexport-runtime",
        "edge-namespace",
        "edge-namespace-call",
        "vitest-nonambiguous-star",
        "vitest-object-call-project",
        "vitest-object-call-arrow-project",
        "vitest-object-call-block-project",
        "vitest-object-call-function-project",
        "vitest-object-call-expression-project",
        "vitest-project-root",
        "namespace-test-options-spread",
        "imported-nested-test-spread",
        "project-test-spread-override",
        "project-constant-spread",
        "nested-local-spread",
        "exported-specifier",
        "default-arrow-block",
        "default-direct-as",
        "default-direct-object",
        "default-direct-satisfies",
        "default-direct-type-assertion",
        "default-exported-const",
        "default-identifier-function",
        "default-arrow",
        "default-as",
        "default-non-null",
        "default-satisfies",
        "default-type-assertion",
    ] {
        assert!(
            edge.iter()
                .any(|project| project.name.as_deref() == Some(name)),
            "missing edge project {name}"
        );
    }
    assert!(edge.iter().any(|project| {
        project.name.as_deref() == Some("imported-nested-test-spread")
            && project.include == vec!["imported-nested-test-spread/**/*.test.ts"]
            && project.exclude == vec!["imported-nested-test-spread/**/*.skip.ts"]
    }));
    assert!(edge.iter().any(|project| {
        project.name.as_deref() == Some("project-test-spread-override")
            && project.include == vec!["project-test-spread-override/**/*.test.ts"]
    }));
    assert!(edge.iter().any(|project| {
        project.name.as_deref() == Some("project-constant-spread")
            && project.include == vec!["project-constant-spread/**/*.test.ts"]
    }));
    assert!(edge.iter().any(|project| {
        project.name.as_deref() == Some("vitest-project-root")
            && project.include == vec!["packages/app/**/*.test.ts"]
            && project.exclude == vec!["packages/app/ignored/**/*.test.ts"]
    }));
    assert!(!edge
        .iter()
        .any(|project| project.name.as_deref() == Some("project-test-spread-local")));
    assert!(edge.iter().any(|project| {
        project.name.as_deref() == Some("nested-local-spread")
            && project.include == vec!["nested-local-spread/**/*.test.ts"]
    }));
    assert!(edge.iter().any(|project| {
        project.name.as_deref() == Some("namespace-test-options-spread")
            && project.include == vec!["namespace-test-options-spread/**/*.test.ts"]
            && project.exclude == vec!["namespace-test-options-spread/**/*.skip.ts"]
    }));

    let project_object_sourced_reexport_path =
        root.join("vitest.project-object-sourced-reexport.mts");
    let project_object_sourced_reexport_source =
        std::fs::read_to_string(&project_object_sourced_reexport_path).unwrap();
    let project_object_sourced_reexport = parse_vitest_fixture(
        &project_object_sourced_reexport_source,
        &project_object_sourced_reexport_path,
        &root,
    )
    .unwrap();
    assert!(project_object_sourced_reexport.iter().any(|project| {
        project.name.as_deref() == Some("vitest-project-object-sourced-reexport")
            && project.include == vec!["vitest-project-object-sourced-reexport/**/*.test.ts"]
    }));

    let project_object_star_reexport_path = root.join("vitest.project-object-star-reexport.mts");
    let project_object_star_reexport_source =
        std::fs::read_to_string(&project_object_star_reexport_path).unwrap();
    let project_object_star_reexport = parse_vitest_fixture(
        &project_object_star_reexport_source,
        &project_object_star_reexport_path,
        &root,
    )
    .unwrap();
    assert!(project_object_star_reexport.iter().any(|project| {
        project.name.as_deref() == Some("vitest-project-object-star-reexport")
            && project.include == vec!["vitest-project-object-star-reexport/**/*.test.ts"]
    }));

    let local_member_spread_path = root.join("vitest.local-member-spread.mts");
    let local_member_spread_source = std::fs::read_to_string(&local_member_spread_path).unwrap();
    let local_member_spread = parse_vitest_fixture(
        &local_member_spread_source,
        &local_member_spread_path,
        &root,
    )
    .unwrap();
    assert!(local_member_spread.iter().any(|project| {
        project.name.as_deref() == Some("vitest-local-member-spread")
            && project.include == vec!["vitest-local-member-spread/**/*.test.ts"]
            && project.exclude == vec!["vitest-local-member-spread/**/*.skip.ts"]
    }));

    assert!(!edge
        .iter()
        .any(|project| project.name.as_deref() == Some("nested-array-should-not-flatten")));
    assert!(!edge
        .iter()
        .any(|project| project.name.as_deref() == Some("vitest-ambiguous-star-a")));
    assert!(!edge
        .iter()
        .any(|project| project.name.as_deref() == Some("vitest-ambiguous-star-b")));
    assert!(!edge
        .iter()
        .any(|project| project.name.as_deref() == Some("vitest-non-spread-call-array")));
    assert!(!edge
        .iter()
        .any(|project| project.name.as_deref() == Some("vitest-non-spread-imported-array")));

    let project_test_override_clears_path = root.join("vitest.project-test-override-clears.mts");
    let project_test_override_clears_source =
        std::fs::read_to_string(&project_test_override_clears_path).unwrap();
    let project_test_override_clears = parse_vitest_fixture(
        &project_test_override_clears_source,
        &project_test_override_clears_path,
        &root,
    )
    .unwrap();
    assert!(project_test_override_clears
        .iter()
        .any(|project| project.name.as_deref() == Some("vitest-project-test-override-clears")));
    assert!(!project_test_override_clears.iter().any(|project| {
        project.name.as_deref() == Some("vitest-project-test-override-clears")
            && project
                .include
                .contains(&"vitest-project-test-override-clears-stale/**/*.test.ts".to_string())
    }));

    let recursive_path = root.join("vitest.recursive.mts");
    let recursive_source = std::fs::read_to_string(&recursive_path).unwrap();
    let recursive = parse_vitest_fixture(&recursive_source, &recursive_path, &root).unwrap();
    assert_eq!(recursive.len(), 1);
    assert_eq!(recursive[0].name, None);

    let spread_test_path = root.join("vitest.spread-test-options.mts");
    let spread_test_source = std::fs::read_to_string(&spread_test_path).unwrap();
    let spread_test = parse_vitest_fixture(&spread_test_source, &spread_test_path, &root).unwrap();
    assert!(spread_test
        .iter()
        .any(|project| project.name.as_deref() == Some("spread-test-options")));

    let invalid_path = root.join("vitest.invalid-project.mts");
    let invalid_source = std::fs::read_to_string(&invalid_path).unwrap();
    let invalid = parse_vitest_fixture(&invalid_source, &invalid_path, &root);
    assert!(invalid.is_err());

    let root_call_spread_local_path = root.join("vitest.root-call-spread-local.mts");
    let root_call_spread_local_source =
        std::fs::read_to_string(&root_call_spread_local_path).unwrap();
    let root_call_spread_local = parse_vitest_fixture(
        &root_call_spread_local_source,
        &root_call_spread_local_path,
        &root,
    )
    .unwrap();
    assert!(root_call_spread_local.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-call-local")
    }));

    let root_member_import_then_export_path =
        root.join("vitest.root-member-import-then-export.mts");
    let root_member_import_then_export_source =
        std::fs::read_to_string(&root_member_import_then_export_path).unwrap();
    let root_member_import_then_export = parse_vitest_fixture(
        &root_member_import_then_export_source,
        &root_member_import_then_export_path,
        &root,
    )
    .unwrap();
    assert!(root_member_import_then_export.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-member-import-then-export")
    }));

    let object_call_import_path = root.join("vitest.object-call-import.mts");
    let object_call_import_source = std::fs::read_to_string(&object_call_import_path).unwrap();
    let object_call_import =
        parse_vitest_fixture(&object_call_import_source, &object_call_import_path, &root).unwrap();
    assert!(object_call_import.iter().any(|project| {
        project.name.as_deref() == Some("vitest-object-call-import")
            && project.include == vec!["vitest-object-call-import/**/*.test.ts"]
    }));

    let root_call_import_path = root.join("vitest.root-call-import.mts");
    let root_call_import_source = std::fs::read_to_string(&root_call_import_path).unwrap();
    let root_call_import =
        parse_vitest_fixture(&root_call_import_source, &root_call_import_path, &root).unwrap();
    assert!(root_call_import.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-call-import")
            && project.include == vec!["vitest-root-call-import/**/*.test.ts"]
    }));

    let member_namespace_star_path = root.join("vitest.member-namespace-star.mts");
    let member_namespace_star_source =
        std::fs::read_to_string(&member_namespace_star_path).unwrap();
    let member_namespace_star = parse_vitest_fixture(
        &member_namespace_star_source,
        &member_namespace_star_path,
        &root,
    )
    .unwrap();
    assert!(member_namespace_star.iter().any(|project| {
        project.name.as_deref() == Some("vitest-member-namespace-star")
            && project.include == vec!["vitest-member-namespace-star/**/*.test.ts"]
    }));

    let root_star_barrel_import_path = root.join("vitest.root-star-barrel-import.mts");
    let root_star_barrel_import_source =
        std::fs::read_to_string(&root_star_barrel_import_path).unwrap();
    let root_star_barrel_import = parse_vitest_fixture(
        &root_star_barrel_import_source,
        &root_star_barrel_import_path,
        &root,
    )
    .unwrap();
    assert!(root_star_barrel_import.iter().any(|project| {
        project.name.as_deref() == Some("vitest-root-star-barrel")
            && project.include == vec!["vitest-root-star-barrel/**/*.test.ts"]
    }));
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
