use super::*;
use crate::integration_tests::project_config::ConfigProjectInput;
use crate::integration_tests::types::Framework;
use std::path::Path;

fn program_projects(
    root: &Path,
    filename: &str,
    source: &str,
    framework: Framework,
) -> anyhow::Result<Vec<crate::integration_tests::types::ConfigProject>> {
    let path = root.join(filename);
    let tsconfig = crate::integration_tests::test_support::tsconfig_without_config(root);
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    crate::integration_tests::runner_config::with_program(&path, source, |program, _| {
        load_config_projects_from_program(
            ConfigProjectInput {
                root,
                framework,
                raw: filename,
                path: &path,
                source,
                config_dir: root,
                resolver: &resolver,
            },
            program,
            None,
        )
    })?
}

#[test]
fn jest_and_dotnet_program_parse_return_no_projects() {
    let root = Path::new("/nm-jest-program-parse");
    for framework in [Framework::Jest, Framework::Dotnet] {
        let projects =
            program_projects(root, "jest.config.js", "module.exports = {};", framework).unwrap();
        assert!(
            projects.is_empty(),
            "{:?} should not parse projects from the oxc program path",
            framework
        );
    }
}

#[test]
fn playwright_program_parse_errors_propagate() {
    let root = Path::new("/nm-playwright-parse-error");
    let error = program_projects(
        root,
        "playwright.config.ts",
        "export default { testMatch: 1 };",
        Framework::Playwright,
    )
    .unwrap_err();
    assert!(error.to_string().contains("expected string literal"));
}
