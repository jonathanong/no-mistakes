use super::*;
use crate::integration_tests::project_config::ConfigProjectInput;
use crate::integration_tests::types::Framework;
use std::path::Path;

#[test]
fn jest_and_dotnet_program_parse_return_no_projects() {
    let root = Path::new("/repo");
    let path = root.join("jest.config.js");
    let source = "module.exports = {};";
    let tsconfig = crate::integration_tests::test_support::tsconfig_without_config(root);
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    for framework in [Framework::Jest, Framework::Dotnet] {
        let projects =
            crate::integration_tests::runner_config::with_program(&path, source, |program, _| {
                load_config_projects_from_program(
                    ConfigProjectInput {
                        root,
                        framework,
                        raw: "jest.config.js",
                        path: &path,
                        source,
                        config_dir: root,
                        resolver: &resolver,
                    },
                    program,
                    None,
                )
            })
            .unwrap()
            .unwrap();
        assert!(
            projects.is_empty(),
            "{:?} should not parse projects from the oxc program path",
            framework
        );
    }
}

#[test]
fn playwright_program_parse_errors_propagate() {
    let root = Path::new("/repo");
    let path = root.join("playwright.config.ts");
    let source = "export default { testMatch: 1 };";
    let tsconfig = crate::integration_tests::test_support::tsconfig_without_config(root);
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    let error = crate::integration_tests::runner_config::with_program(&path, source, |program, _| {
        load_config_projects_from_program(
            ConfigProjectInput {
                root,
                framework: Framework::Playwright,
                raw: "playwright.config.ts",
                path: &path,
                source,
                config_dir: root,
                resolver: &resolver,
            },
            program,
            None,
        )
    })
    .unwrap()
    .unwrap_err();
    assert!(error.to_string().contains("expected string literal"));
}
