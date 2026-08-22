use super::test_config;
use super::ConfigProjectInput;
use crate::integration_tests::types::{ConfigProject, Framework};
use anyhow::Result;
use oxc_ast::ast::Program;

pub(in crate::integration_tests) fn load_config_projects_from_program(
    input: ConfigProjectInput<'_>,
    program: &Program<'_>,
    _visible_files: Option<&crate::fx::PathSet>,
) -> Result<Vec<ConfigProject>> {
    let ConfigProjectInput {
        root,
        framework,
        raw,
        path,
        source,
        config_dir,
        resolver,
    } = input;
    match framework {
        Framework::Playwright => {
            let parsed = test_config::playwright::parse_program_with_resolver(
                program, source, path, config_dir, resolver,
            )?;
            Ok(parsed.into_projects(root, raw))
        }
        Framework::Vitest => {
            let workspace = test_config::vitest::is_vitest_project_array_path(path);
            let parsed = test_config::vitest::parse_program_with_resolver(
                program, source, path, config_dir, root, resolver,
            )?;
            Ok(parsed
                .into_iter()
                .map(|mut project| {
                    project.config = Some(raw.to_string());
                    project.workspace = workspace;
                    project
                })
                .collect())
        }
        _ => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests;
