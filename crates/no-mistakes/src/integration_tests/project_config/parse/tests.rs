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
