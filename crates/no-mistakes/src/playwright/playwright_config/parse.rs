use super::ast_nav::{default_export_object, project_objects, top_level_object_bindings};
use super::merge::{merge_project, parse_options};
use super::types::{ParsedOptions, PlaywrightConfig};
use crate::playwright::ast;
use anyhow::Result;
use oxc_ast::ast::Program;
use std::path::Path;
pub fn parse(source: &str, config_dir: &Path) -> Result<PlaywrightConfig> {
    parse_from_path(source, Path::new("playwright.config.ts"), config_dir)
}

pub fn parse_from_path(source: &str, path: &Path, config_dir: &Path) -> Result<PlaywrightConfig> {
    ast::with_program(path, source, |program, source| {
        parse_program(program, source, path, config_dir)
    })?
}

fn parse_program(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    config_dir: &Path,
) -> Result<PlaywrightConfig> {
    let (config, failures) = parse_program_partial(program, source, path, config_dir)?;
    if let Some((_, error)) = failures.into_iter().next() {
        return Err(error);
    }
    Ok(config)
}

pub(super) type ProjectFailures = Vec<(Option<String>, anyhow::Error)>;
pub(super) fn parse_partial(
    source: &str,
    path: &Path,
    config_dir: &Path,
) -> Result<(PlaywrightConfig, ProjectFailures)> {
    ast::with_program(path, source, |program, source| {
        parse_program_partial(program, source, path, config_dir)
    })?
}

fn parse_program_partial(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    config_dir: &Path,
) -> Result<(PlaywrightConfig, ProjectFailures)> {
    let bindings = top_level_object_bindings(program);
    let Some(root_object) = default_export_object(program) else {
        return Ok((
            PlaywrightConfig {
                name: None,
                projects: vec![merge_project(config_dir, &ParsedOptions::default(), None)],
            },
            Vec::new(),
        ));
    };
    let root_options = parse_options(root_object, source, path, &bindings)?;
    let project_objects = project_objects(root_object);

    if project_objects.is_empty() {
        return Ok((
            PlaywrightConfig {
                name: root_options.name.clone(),
                projects: vec![merge_project(config_dir, &root_options, None)],
            },
            Vec::new(),
        ));
    }

    let mut projects = Vec::new();
    let mut failures = Vec::new();
    for project_object in project_objects {
        match parse_options(project_object, source, path, &bindings) {
            Ok(options) => projects.push(merge_project(config_dir, &root_options, Some(options))),
            Err(error) => {
                let name = super::ast_nav::property_expression(project_object, "name")
                    .and_then(|value| super::literals::optional_string(value, source));
                failures.push((name, error));
            }
        }
    }
    Ok((
        PlaywrightConfig {
            name: root_options.name,
            projects,
        },
        failures,
    ))
}
